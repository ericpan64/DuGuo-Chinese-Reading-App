/*
/// Module for html rendering.
/// 
/// html.rs
/// └── pub fn:
///     └── convert_string_to_tokenized_html
///     └── render_document_table
///     └── render_vocab_table
*/

use crate::{
    connect_to_redis,
    config::{TOKENIZER_HOSTNAME, TOKENIZER_PORT, USER_DOC_COLL_NAME, USER_VOCAB_COLL_NAME},
    models::{
        user::{User, UserDoc, UserVocab},
        zh::{CnType, CnPhonetics, CnEnDictEntry}
    }
};
use mongodb::{
    bson::{doc, Bson, from_bson},
    sync::Database
};
use regex::Regex;
use std::{
    io::prelude::*,
    net::TcpStream
};

/// Renders the HTML using the given CnType and CnPhonetics.
/// Note: the tokenizer only returns pinyin, however that's used to lookup the CEDICT entry.
/// From the CEDICT entry, the specified CnType, CnPhonetics are rendered.
pub async fn convert_string_to_tokenized_html(s: &str, cn_type: &CnType, cn_phonetics: &CnPhonetics) -> String {
    const PHRASE_DELIM: char = '$';
    const PINYIN_DELIM: char = '`';
    let mut conn = connect_to_redis().await.unwrap();
    let tokenized_string = tokenize_string(s.to_string()).expect("Tokenizer connection error");
    let n_phrases = tokenized_string.matches(PHRASE_DELIM).count();
    // Estimate pre-allocated size: max ~2100 chars per phrase (conservitively 2500), 1 usize per char
    let mut res = String::with_capacity(n_phrases * 2500);
    for token in tokenized_string.split(PHRASE_DELIM) {
        let token_vec: Vec<&str> = token.split(PINYIN_DELIM).collect();
        let phrase = token_vec[0]; // Simplified if Chinese
        let raw_pinyin = token_vec[1];
        // Skip lookup for phrases with no Chinese chars
        if is_english_phrase(phrase) || has_chinese_punctuation(phrase) {
            // handle newlines, else render word aligned with other text
            if phrase.contains('\n') {
                res += &phrase.replace('\n', "<br>");
            } else {
                let mut new_phrase = String::with_capacity(250);
                new_phrase += "<span><table style=\"display: inline-table; text-align: center;\"><tr><td></td></tr><tr><td>";
                new_phrase += &phrase.replace('\n', "<br>");
                new_phrase += "</td></tr></table></span>";
                res += &new_phrase;
            }
        } else {
            // For each phrase, lookup as CnEnDictEntry
            let entry = CnEnDictEntry::from_tokenizer_components(&mut conn, phrase, raw_pinyin).await;
            if entry.lookup_failed() {
                res += generate_html_for_not_found_phrase(phrase).as_str();
            } else {
                match (cn_type, cn_phonetics) {
                    (CnType::Traditional, CnPhonetics::Pinyin) => res += entry.trad_html.as_str(),
                    (CnType::Traditional, CnPhonetics::Zhuyin) => res += entry.trad_zhuyin_html.as_str(),
                    (CnType::Simplified, CnPhonetics::Pinyin) => res += entry.simp_html.as_str(),
                    (CnType::Simplified, CnPhonetics::Zhuyin) => res += entry.simp_zhuyin_html.as_str(),
                }
            }
        }
    }
    return res;
}

pub fn render_document_table(db: &Database, username: &str) -> String {
    // get all documents for user
    const TRASH_ICON: &str = "https://icons.getbootstrap.com/icons/trash.svg";
    let coll = (*db).collection(USER_DOC_COLL_NAME);
    let (cn_type, cn_phonetics) = User::get_user_settings(db, username);
    let mut res = String::new();
    res += "<table id=\"doc-table\" class=\"table table-hover\">\n";
    res += "<thead class=\"table-light\">\n<tr><th>Title</th><th>Source</th><th>Created On (UTC)</th><th>Delete</th></tr>\n";
    res += "</thead>\n";
    let query_doc = doc! { "username": username, "cn_type": cn_type.as_str(), "cn_phonetics": cn_phonetics.as_str() };
    match coll.find(query_doc, None) {
        Ok(cursor) => {
            // add each document as a <tr> item
            res += "<tbody>\n";
            let url_re = Regex::new(r"^(http{1}s?://)?(([a-zA-z0-9])+\.)+([a-zA-z0-9]*)(/{1}.*)?$").unwrap();
            for item in cursor {
                // unwrap BSON document
                let user_doc = item.unwrap();
                let UserDoc { title, created_on, source, .. } = from_bson(Bson::Document(user_doc)).unwrap(); 
                let delete_button = format!("<a href=\"/api/delete-doc/{}\"><img src={}></img></a>", &title, TRASH_ICON);
                let title = format!("<a href=\"/u/{}/{}\">{}</a>", &username, &title, &title);
                // only format as link if it's a URL
                let source = match url_re.is_match(&source) {
                    true => format!("<a href=\"{}\" target=\"_blank\">Link</a>", source),
                    false => {
                        match source.as_str() {
                            "" => String::from("n/a"),
                            _ => source
                        }
                    }
                };
                res += format!("<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n", title, source, &created_on[0..10], delete_button).as_str();
            }
            res += "</tbody>\n";
        },
        Err(e) => { eprintln!("Error when searching for documents for user {}: {:?}", username, e); }
    }
    res += "<caption hidden>List of your saved vocabulary.</caption>\n</table>";
    return res;
}

pub fn render_vocab_table(db: &Database, username: &str) -> String {
    const TRASH_ICON: &str = "https://icons.getbootstrap.com/icons/trash.svg";
    let coll = (*db).collection(USER_VOCAB_COLL_NAME);
    let (cn_type, cn_phonetics) = User::get_user_settings(db, username);
    let mut res = String::new();
    res += "<table id=\"vocab-table\" class=\"table table-hover\">\n";
    res += "<thead class=\"table-light\">\n<tr><th>Phrase</th><th>Saved From (plaintext)</th><th>Saved On (UTC)</th><th>Delete</th></tr>\n";
    res += "</thead>\n";
    let query_doc = doc! { "username": username, "cn_type": cn_type.as_str(), "cn_phonetics": cn_phonetics.as_str() };
    match coll.find(query_doc, None) {
        Ok(cursor) => {
            res += "<tbody>\n";
            // add each document as a <tr> item
            for item in cursor {
                // unwrap BSON document
                let user_doc = item.unwrap();
                let UserVocab { uid, from_doc_title, phrase, phrase_html, created_on, .. } = from_bson(Bson::Document(user_doc)).unwrap();
                let delete_button = format!("<a href=\"/api/delete-vocab/{}\"><img src={}></img></a>", phrase, TRASH_ICON);
                let phrase_html = remove_download_link_from_phrase_html(phrase_html, &uid);
                let row = format!("<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n", phrase_html, &from_doc_title, &created_on[0..10], &delete_button);
                res += &row;
            }
            res += "</tbody>\n";
        },
        Err(e) => { eprintln!("Error when searching for vocab for user {}: {:?}", username, e); }
    }
    res += "<caption hidden>List of your saved documents.</caption>\n</table>";
    return res;
}

/// Assumes that the download link in phrase_html is the following format:
///  <a role=&quot;button&quot; href=&quot;#{uid}&quot;><img src=&quot;https://icons.getbootstrap.com/icons/download.svg&quot;></img></a>
/// Note: there is a single space in front of the link (which also gets removed).
pub fn remove_download_link_from_phrase_html(phrase_html: String, uid: &str) -> String {
    let download_link = format!(" <a role=&quot;button&quot; href=&quot;#{}&quot;><img src=&quot;https://icons.getbootstrap.com/icons/download.svg&quot;></img></a>", uid);
    let res = phrase_html.replace(&download_link, "");
    return res;
}

/// Connect to tokenizer service and tokenizes the string. The delimiters are $ and ` since neither character appears in CEDICT.
/// The format of the string is: "phrase1`raw_pinyin`formatted_pinyin$phrase2`raw_pinyin2`formatted_pinyin2$ ..."
/// Sleeps 1sec after write and 1sec after read due to a strange issue where data inconsistently stopped writing (probably async weirdness)
fn tokenize_string(mut s: String) -> std::io::Result<String> {
    s = s.replace("  ", ""); // remove excess whitespace for tokenization, keep newlines. "  " instead of " " to preserve non-Chinese text
    let mut stream = TcpStream::connect(format!("{}:{}", TOKENIZER_HOSTNAME, TOKENIZER_PORT))?;
    stream.write_all(s.as_bytes())?;
    let mut header_bytes = [0; 64];
    stream.read_exact(&mut header_bytes)?;
    let n_bytes: usize = String::from_utf8(header_bytes.to_vec()).unwrap()
        .trim().parse::<usize>().unwrap();
    let mut tokenized_bytes = vec![0; n_bytes];
    stream.read_exact(&mut tokenized_bytes)?;
    let res = String::from_utf8(tokenized_bytes).unwrap();
    return Ok(res);
}

fn is_english_phrase(s: &str) -> bool {
    // English chars use 1 byte, Chinese chars use 3 bytes
    return s.len() == s.chars().count();
}

fn has_chinese_punctuation(s: &str) -> bool {
    // Chinese punctuation is a Chinese char, however shouldn't be rendered as such
    const PUNCT: [char; 15] = ['（', '）', '“', '”', '、', '，', '。', '《', '》', '：', '！', '？','￥', '—', '；'];
    let mut res = false;
    for c in s.chars() {
        if PUNCT.contains(&c) {
            res = true;
            break;
        }
    }
    return res;
}

fn generate_html_for_not_found_phrase(phrase: &str) -> String {
    let mut res = String::with_capacity(2500); // Using ~2500 characters as conservative estimate
    res += "<span tabindex=\"0\" data-bs-toggle=\"popover\" data-bs-trigger=\"focus\" data-bs-content=\"Phrase not found in database.\">";
    res += "<table style=\"display: inline-table;\">";
    res += "<tr></tr>"; // No pinyin found
    let mut phrase_td = String::with_capacity(10 * phrase.len()); // Adding ~10 chars per 3 bytes (1 chinese character), so this is conservative
    for c in phrase.chars() {
        phrase_td += format!("<td>{}</td>", c).as_str();
    }
    res += format!("<tr>{}</tr>", phrase_td).as_str();
    res += "</table>";
    res += "</span>";
    return res;
}