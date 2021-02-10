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
    CacheItem,
    connect_to_redis,
    config::{TOKENIZER_HOSTNAME, TOKENIZER_PORT, USER_DOC_COLL_NAME, USER_VOCAB_COLL_NAME},
    models::{
        user::{User, UserDoc, UserVocab},
        zh::{CnType, CnPhonetics, CnEnDictEntry}
    }
};
use mongodb::{
    bson::{self, doc, Bson},
    sync::Database
};
use regex::Regex;
use std::{
    io::prelude::*,
    net::TcpStream
};

/* Public Functions */
/// Organizes data from CnEnDictEntry, then renders the appropriate HTML.
pub fn render_phrase_html(entry: &CnEnDictEntry, cn_type: &CnType, cn_phonetics: &CnPhonetics, include_sound_link: bool, include_download_link: bool) -> String {
    let (phrase, char_list): (&str, Vec<char>) = match cn_type {
        CnType::Traditional => (&entry.trad, entry.trad.chars().collect()),
        CnType::Simplified => (&entry.simp, entry.simp.chars().collect())
    };
    // Rendering Helper Fn (keep as closure to retain context)
    let perform_phrase_render = |phrase: &str, phonetic_str: &str, char_list: Vec<char>, phonetic_list: Vec<&str>| -> String {
        const SOUND_ICON: &str = "https://icons.getbootstrap.com/icons/volume-up-fill.svg";
        const DOWNLOAD_ICON: &str = "https://icons.getbootstrap.com/icons/download.svg";
        let mut res = String::with_capacity(2500);
        // Start <span> (popup config)
        res += format!("<span class=\"{}\" tabindex=\"0\"", entry.uid).as_str();
        res += format!(" data-bs-toggle=\"popover\" data-bs-content=\"{}\"", format_defn_html(entry)).as_str();
        res += format!(" title=\"{} [{}]", phrase, phonetic_str).as_str();
        if include_sound_link {
            res += format!(" <a role=&quot;button&quot; href=&quot;#~{}&quot;>", phrase).as_str();
            res += format!("<img src=&quot;{}&quot;></img>", SOUND_ICON).as_str();
            res += "</a>";
        }
        if include_download_link {
            res += format!(" <a role=&quot;button&quot; href=&quot;#{}&quot;>", entry.uid).as_str();
            res += format!("<img src=&quot;{}&quot;></img>", DOWNLOAD_ICON).as_str();
            res += "</a>";
        }
        res += "\"";
        res += " data-bs-html=\"true\">";
        // Start <table> entry (phrase with phonetics)
        // add phonetic row
        res += "<table>";
        res += "<tr>";
        for i in 0..char_list.len() {
            res += format!("<td class=\"phonetic\" name=\"{}\">", char_list[i]).as_str();
            res += phonetic_list[i];
            res += "</td>";
        }
        res += "</tr>";
        // add phrase row
        res += "<tr>";
        for i in 0..char_list.len() {
            res += "<td class=\"char\">";
            res += &char_list[i].to_string();
            res += "</td>";
        }
        res += "</tr>";
        res += "</table>";
        res += "</span>";
        return res;
    };
    let res = match cn_phonetics {
        CnPhonetics::Pinyin => {
            let pinyin_list: Vec<&str> = entry.formatted_pinyin.split(' ').collect();
            perform_phrase_render(phrase, &entry.raw_pinyin, char_list, pinyin_list)
        },
        CnPhonetics::Zhuyin => {
            let zhuyin_list: Vec<&str> = entry.zhuyin.split(' ').collect();
            perform_phrase_render(phrase, &entry.zhuyin, char_list, zhuyin_list)
        }
    };
    return res;
}

/// Renders the HTML using the given CnType and CnPhonetics.
/// Refer to tokenizer_string() for formatting details.
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
        let phrase = token_vec[0]; // If Chinese, then Simplified
        let raw_pinyin = token_vec[1];
        let uid = CnEnDictEntry::generate_uid(vec![phrase,raw_pinyin]);
        // Skip lookup for phrases with no Chinese chars
        if is_english_phrase(phrase) || has_chinese_punctuation(phrase) {
            // handle newlines, else render word aligned with other text
            if phrase.contains('\n') {
                res += &phrase.replace('\n', "<br>");
            } else {
                let mut new_phrase = String::with_capacity(250);
                new_phrase += "<span><table><tr><td></td></tr><tr><td>";
                new_phrase += &phrase.replace('\n', "<br>");
                new_phrase += "</td></tr></table></span>";
                res += &new_phrase;
            }
        } else {
            // For each phrase, lookup as CnEnDictEntry
            let entry = CnEnDictEntry::from_uid(&mut conn, uid).await;
            if entry.lookup_failed() {
                res += generate_html_for_not_found_phrase(phrase).as_str();
            } else {
                res += render_phrase_html(&entry, cn_type, cn_phonetics, true, true).as_str();
            }
        }
    }
    return res;
}

/// Renders the UserDoc table for userprofile.html.tera.
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
                let UserDoc { title, created_on, source, .. } = bson::from_bson(Bson::Document(user_doc)).unwrap(); 
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

/// Renders the UserVocab table for userprofile.html.tera.
pub fn render_vocab_table(db: &Database, username: &str) -> String {
    const TRASH_ICON: &str = "https://icons.getbootstrap.com/icons/trash.svg";
    let coll = (*db).collection(USER_VOCAB_COLL_NAME);
    let (cn_type, cn_phonetics) = User::get_user_settings(db, username);
    let mut res = String::new();
    res += "<table id=\"vocab-table\" class=\"table table-hover\">\n";
    res += "<thead class=\"table-light\">\n<tr><th>Phrase</th><th>Saved From</th><th>Radical Map</th><th>Saved On (UTC)</th><th>Delete</th></tr>\n";
    res += "</thead>\n";
    let query_doc = doc! { "username": username, "cn_type": cn_type.as_str(), "cn_phonetics": cn_phonetics.as_str() };
    match coll.find(query_doc, None) {
        Ok(cursor) => {
            res += "<tbody>\n";
            // add each document as a <tr> item
            for item in cursor {
                // unwrap BSON document
                let user_doc = item.unwrap();
                let UserVocab { uid, from_doc_title, phrase_html, created_on, radical_map, .. } = bson::from_bson(Bson::Document(user_doc)).unwrap();
                let from_doc_title = format!("<a href=\"{}/{}\">{}</a>", username, from_doc_title, from_doc_title);
                let delete_button = format!("<a href=\"/api/delete-vocab/{}\"><img src={}></img></a>", uid, TRASH_ICON);
                let row = format!("<tr><td>{}</td><td>{}</td><td style\"white-space: pre\">{}</td><td>{}</td><td>{}</td></tr>\n", phrase_html, &from_doc_title, radical_map, &created_on[0..10], &delete_button);
                res += &row;
            }
            res += "</tbody>\n";
        },
        Err(e) => { eprintln!("Error when searching for vocab for user {}: {:?}", username, e); }
    }
    res += "<caption hidden>List of your saved documents.</caption>\n</table>";
    return res;
}

/* Private Functions */
/// Connect to tokenizer service and tokenizes the string. The delimiters are $ and ` since neither character appears in CEDICT.
/// The format of the string is: "phrase1`raw_pinyin$phrase2`raw_pinyin2$ ..."
/// The string is written to the TCP stream until completion.
/// From the tokenizer, 2 messages are sent: 
///     1) A u64 (as bytes) indicating the size of the tokenizer results
///     2) The tokenizer result string (as bytes)
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

/// Formats the definition in the phrase HTML. Used in render_phrase_html().
fn format_defn_html(entry: &CnEnDictEntry) -> String {
    const DEFN_DELIM: char = '/'; // Used to separate the description for a single concept definition
    const MULTI_DEFN_DELIM: char = '$'; // Used when a single phrase represents multiple different concepts
    let mut res = String::with_capacity(2500);
    let all_defns: Vec<&str> = entry.defn.split(MULTI_DEFN_DELIM).collect();
    for (i, defn) in all_defns.iter().enumerate() {
        let defn = &defn[1..defn.len()-1];
        let defn = defn.replace("\"", "\'");
        for (j, li) in defn.split(DEFN_DELIM).enumerate() {
            res += format!("{}. {}", j+1, li).as_str();
            if j != defn.len() - 1 {
                res += "<br>";
            } else if i != all_defns.len() - 1 {
                res += "<hr>"
            }
        }
    }
    return res;
}

/// A weak check to distinguish if a phrase is English.
/// English chars use 1 byte, Chinese chars use 3 bytes.
fn is_english_phrase(s: &str) -> bool {
    return s.len() == s.chars().count();
}

/// A weak check to distinguish phrases with Chinese puntuation.
/// Chinese punctuation is a Chinese char (3 bytes) that should be skipped in processing.
fn has_chinese_punctuation(s: &str) -> bool {
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

/// Generates generic HTML with a "Phrase not found in database" popup.
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