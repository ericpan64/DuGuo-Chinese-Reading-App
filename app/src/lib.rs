/// Helper Functions
/// 
/// Structure:
/// 
/// duguo (lib.rs)
/// ├── connect_to_mongodb: Fn
/// ├── convert_rawstr_to_string: Fn
/// ├── check_password: Fn
/// |
/// ├── DatabaseItem: Trait
/// ├── {CnType, CnPhonetics}: Enums
/// ├── {User, SandboxDoc, UserDoc, CnEnDictEntry, UserVocab, UserVocabList, UserFeedback}: Structs
/// │   └── Impl DatabaseItem for all Structs
/// |   └── Impl for all Structs
/// |
/// ├── html_rendering: Module
/// └── cookie_handling: Module

/// FYI: there is a private "config.rs" file where I define and implement the some of the functionality.
/// A "dummy" config.rs is provided as a reference, though notably the values don't match production.
/// For security reasons, the production values are kept private.
mod config;
use config::*;
use futures::StreamExt;
use tokio::runtime::Handle;
use mongodb::{
    bson::{doc, Bson, document::Document, to_document, from_bson, to_bson},
    error::Error,
    Client, Database
};
use rocket::http::RawStr;
use std::{
    io::prelude::*,
    net::TcpStream,
};
use serde::{Serialize, Deserialize};
use uuid::Uuid;


/* Public Functions */
/// Connectivity
pub fn connect_to_mongodb(rt: &Handle) -> Result<Database, Error> {
    let client = (*rt).block_on(Client::with_uri_str(DB_URI))?;
    let db: Database = client.database(DB_NAME);
    return Ok(db);
}

/// Security
pub fn convert_rawstr_to_string(s: &RawStr) -> String {
    let mut res = match s.url_decode() {
        Ok(val) => val,
        Err(e) => {
            println!("UTF-8 Error encountered, returning empty String. Err: {:?}", e);
            String::new()
        }
    };
    // Note: can't sanitize '/' since that breaks default character encoding
    res = res.replace(&['<', '>', '(', ')', '!', '\"', '\'', '\\', ';', '{', '}', ':'][..], "");
    return res;
}

pub async fn check_password(db: &Database, username: &str, pw_to_check: &str) -> bool {
    let coll = (*db).collection(USER_COLL_NAME);
    let hashed_pw = str_to_hashed_string(pw_to_check);
    let query_doc = doc! { "username": username, "pw_hash": &hashed_pw };
    let res = match coll.find_one(query_doc, None).await.unwrap() {
        Some(document) => {
            let saved_hash = document.get("pw_hash").and_then(Bson::as_str).expect("No password was stored");
            saved_hash == &hashed_pw
        },
        None => false
    };
    return res;
}

/* Traits */
pub trait DatabaseItem {
    // Implemented Defaults
    fn as_document(&self) -> Document where Self: Serialize {
        return to_document(self).unwrap();
    }
    fn as_bson(&self) -> Bson where Self: Serialize {
        return to_bson(self).unwrap();
    }
    fn try_insert(&self, db: &Database, rt: &Handle) -> Result<String, Error> where Self: Serialize {
        let coll = (*db).collection(self.collection_name());
        let new_doc = self.as_document();
        match (*rt).block_on(coll.insert_one(new_doc, None)) {
            Ok(_) => {}
            Err(e) => { return Err(e); }
        }
        return Ok(self.primary_key().to_string());
    }
    fn try_update(&self, db: &Database, rt: &Handle, key: &str, new_value: &str) -> Result<String, Error> where Self: Serialize {
        let coll = (*db).collection(self.collection_name());
        let update_doc = doc! { key: new_value };
        let update_query = doc! { "$set": update_doc };
        match (*rt).block_on(coll.update_one(self.as_document(), update_query, None)) {
            Ok(_) => {},
            Err(e) => { return Err(e); }
        }
        return Ok(self.primary_key().to_string());
    }

    // Requires Implementation
    fn collection_name(&self) -> &str;
    fn primary_key(&self) -> &str;
}

/* Enums */
#[derive(Serialize, Deserialize, Clone, Debug)]
enum CnType {
    Traditional,
    Simplified
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum CnPhonetics {
    Pinyin,
    Zhuyin
}

/* Structs */
#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    username: String,
    pw_hash: String,
    email: String,
    cn_type: CnType,
    cn_phonetics: CnPhonetics,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SandboxDoc {
    doc_id: String,
    body: String,
    body_html: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserDoc {
    username: String,
    title: String,
    body: String,
    body_html: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CnEnDictEntry {
    trad: String,
    simp: String,
    raw_pinyin: String,
    pub formatted_pinyin: String,
    trad_html: String,
    simp_html: String,
    def: String,    
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserVocab {
    username: String,
    from_doc_title: String,
    phrase: CnEnDictEntry,
    cn_type: CnType,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserVocabList {
    username: String,
    unique_phrase_list: String, // comma-delimited
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserFeedback {
    feedback: String,
    contact: String, // "" if none provided
    datetime: String, // formatted in JS
}

/* Trait Implementation */
impl DatabaseItem for User {
    fn try_insert(&self, db: &Database, rt: &Handle) -> Result<String, Error> {
        let coll = (*db).collection(USER_COLL_NAME);
        let can_register = (*rt).block_on(User::check_if_username_and_email_are_available(db, &self.username, &self.email));
        let mut message = String::new();
        if can_register {
            let new_doc = self.as_document();
            match (*rt).block_on(coll.insert_one(new_doc, None)) {
                Ok(_) => {
                    let success_msg = format!("Registration successful! Username: {}", self.username);
                    message.push_str(&success_msg);
                }
                Err(e) => { return Err(e); }
            }
        }
        return Ok(message);
    }
    fn collection_name(&self) -> &str { return USER_COLL_NAME; }
    fn primary_key(&self) -> &str { return &self.username; }
}

impl DatabaseItem for SandboxDoc {
    fn collection_name(&self) -> &str { return SANDBOX_COLL_NAME; }
    fn primary_key(&self) -> &str { return &self.doc_id; }
}

impl DatabaseItem for UserDoc {
    fn collection_name(&self) -> &str { return USER_DOC_COLL_NAME; }
    /// NOTE: this is not unique per document, a unique primary_key is username + title.
    fn primary_key(&self) -> &str { return &self.username; }
}

impl DatabaseItem for CnEnDictEntry {
    fn collection_name(&self) -> &str { return CEDICT_COLL_NAME; }
    fn primary_key(&self) -> &str { return &self.trad; }
}

impl DatabaseItem for UserVocab {
    fn try_insert(&self, db: &Database, rt: &Handle) -> Result<String, Error> where Self: Serialize {
        let coll = (*db).collection(self.collection_name());
        let new_doc = self.as_document();
        match (*rt).block_on(coll.insert_one(new_doc, None)) {
            Ok(_) => {
                UserVocabList::append_to_user_vocab_list(db, rt, &self.username, &self.phrase)?;
            },
            Err(e) => { return Err(e); }
        }
        return Ok(self.primary_key().to_string());
    }

    fn collection_name(&self) -> &str { return USER_VOCAB_COLL_NAME; }
    fn primary_key(&self) -> &str { return &self.phrase.trad; } // phrase.simp is also a unique, primary key
}

impl DatabaseItem for UserVocabList {
    fn collection_name(&self) -> &str { return USER_VOCAB_LIST_COLL_NAME; }
    fn primary_key(&self) -> &str { return &self.username; }
}

impl DatabaseItem for UserFeedback {
    fn collection_name(&self) -> &str { return USER_FEEDBACK_COLL_NAME; }
    fn primary_key(&self) -> &str { return &self.datetime; }
}

/* Struct Implementation */
impl User {
    fn default_settings() -> (CnType, CnPhonetics) {
        return (CnType::Simplified, CnPhonetics::Pinyin);
    }

    pub fn new(username: String, password: String, email: String) -> Self {
        let pw_hash = str_to_hashed_string(&password);
        let (cn_type, cn_phonetics) = User::default_settings();
        let new_user = User { username, pw_hash, email, cn_type, cn_phonetics };
        return new_user;
    }

    pub async fn check_if_username_exists(db: &Database, username: &str) -> bool {
        let coll = (*db).collection(USER_COLL_NAME);
        return (coll.find_one(doc! {"username": username }, None).await.unwrap()) != None;
    }

    async fn get_user_settings(db: &Database, username: &str) -> (CnType, CnPhonetics) {
        let coll = (*db).collection(USER_COLL_NAME);
        let res_tup = match coll.find_one(doc! {"username": username }, None).await.unwrap() {
            Some(user_doc) => {
                let User { cn_type, cn_phonetics, ..} = from_bson(Bson::Document(user_doc)).unwrap();
                (cn_type, cn_phonetics)
            },
            None => { User::default_settings() }
        };
        return res_tup;
    }

    async fn check_if_username_and_email_are_available(db: &Database, username: &str, email: &str) -> bool {
        let coll = (*db).collection(USER_COLL_NAME);
        let username_query = coll.find_one(doc! {"username": username }, None).await.unwrap();
        let email_query = coll.find_one(doc! {"email": email}, None).await.unwrap();
        return (username_query == None) && (email_query == None);
    }
}

impl SandboxDoc {
    pub async fn new(db: &Database, body: String) -> Self {
        let doc_id = Uuid::new_v4().to_string();
        let body_html = html_rendering::convert_string_to_tokenized_html(db, &body).await;
        let new_doc = SandboxDoc { doc_id, body, body_html };
        return new_doc;
    }

    pub async fn find_doc_from_id(db: &Database, doc_id: String) -> Option<String> {
        let coll = (*db).collection(SANDBOX_COLL_NAME);
        let query_doc = doc! { "doc_id": doc_id };
        let res = match coll.find_one(query_doc, None).await.unwrap() {
            Some(doc) => Some(doc.get("body").and_then(Bson::as_str).expect("No body was stored").to_string()),
            None => None
        };
        return res;
    }
}

impl UserDoc {
    pub async fn new(db: &Database, username: String, desired_title: String, body: String) -> Self {
        let body_html = html_rendering::convert_string_to_tokenized_html(db, &body).await;
        // If title is non-unique, try appending digits until match
        let coll = (*db).collection(USER_DOC_COLL_NAME);
        let mut title_exists = (coll.find_one(doc! {"username": &username, "title": &desired_title}, None).await.unwrap()) != None;
        let title = match title_exists {
            true => {
                // Try new titles until unique one found
                let mut count = 0;
                let mut new_title = String::new();
                while title_exists {
                    count += 1;
                    let appended = format!("-{}", count);
                    new_title = desired_title.clone() + appended.as_str(); // need .clone() here because of loop
                    title_exists = (coll.find_one(doc! {"username": &username, "title": &new_title}, None).await.unwrap()) != None;
                }
                new_title
            },
            false => { desired_title }
        };
        let new_doc = UserDoc { username, title, body, body_html };
        return new_doc;
    }

    pub async fn get_body_html_from_user_doc(db: &Database, username: &str, title: &str) -> Option<String> {
        let coll = (*db).collection(USER_DOC_COLL_NAME);
        let query_doc = doc! { "username": username, "title": title };
        let doc_body = match coll.find_one(query_doc, None).await.unwrap() {
            Some(doc) => Some(doc.get("body_html").and_then(Bson::as_str).unwrap().to_string()),
            None => None
        };
        return doc_body;
    }

    pub async fn try_delete(db: &Database, username: &str, title: &str) -> bool {
        let coll = (*db).collection(USER_DOC_COLL_NAME);
        let query_doc = doc! { "username": username, "title": title }; 
        let res = match coll.delete_one(query_doc, None).await {
            Ok(delete_res) => delete_res.deleted_count == 1,
            Err(_) => false,
        };
        return res;
    }
}

impl CnEnDictEntry {
    pub async fn new(db: &Database, phrase_str: &str) -> Self {
        // Try simplified, then traditional
        let res = match CnEnDictEntry::lookup_phrase(db, "simp", phrase_str).await {
            Some(obj) => obj,
            None => {
                match CnEnDictEntry::lookup_phrase(db, "trad", phrase_str).await {
                    Some(obj) => obj,
                    None => CnEnDictEntry::generate_empty_phrase()
                }
            }
        };
        return res;
    }

    // TODO: refactor this to apply for DatabaseItem trait
    pub async fn lookup_phrase(db: &Database, key: &str, value: &str) -> Option<Self> {
        let coll = (*db).collection(CEDICT_COLL_NAME);
        let query_doc = doc! { key: value };
        let res: Option<Self> = match coll.find_one(query_doc, None).await.unwrap() {
            Some(doc) => Some(from_bson(Bson::Document(doc)).unwrap()),
            None => None,
        };
        return res;
    }

    pub fn generate_empty_phrase() -> Self {
        // TODO: find a cleaner alternative to not found vocab
        const LOOKUP_ERROR_MSG: &str = "N/A - Not found in database";
        let res = CnEnDictEntry {
            trad: String::from(LOOKUP_ERROR_MSG),
            simp: String::from(LOOKUP_ERROR_MSG),
            raw_pinyin: String::from(LOOKUP_ERROR_MSG),
            formatted_pinyin: String::from(LOOKUP_ERROR_MSG),
            def: String::from(LOOKUP_ERROR_MSG),
            trad_html: String::from(LOOKUP_ERROR_MSG),
            simp_html: String::from(LOOKUP_ERROR_MSG),
        };
        return res;
    }
}

impl UserVocab {
    pub async fn new(db: &Database, username: String, saved_phrase: String, from_doc_title: String) -> Self {
        // Try simplified, then traditional
        let mut cn_type = CnType::Simplified;
        let phrase: CnEnDictEntry = match CnEnDictEntry::lookup_phrase(db, "simp", &saved_phrase).await {
            Some(sp) => sp,
            None => {
                cn_type = CnType::Traditional;
                match CnEnDictEntry::lookup_phrase(db, "trad", &saved_phrase).await {
                    Some(tp) => tp,
                    None => CnEnDictEntry::generate_empty_phrase()
                }
            }
        };
        let new_vocab = UserVocab { username, from_doc_title, phrase, cn_type };
        return new_vocab;
    }

    pub fn try_delete(db: &Database, rt: &Handle, username: &str, phrase: CnEnDictEntry) -> bool {
        let coll = (*db).collection(USER_VOCAB_COLL_NAME);
        let query_doc = doc! { "username": username, "phrase.trad": &phrase.trad, "phrase.simp": &phrase.simp };
        let mut res = match (*rt).block_on(coll.delete_one(query_doc, None)) {
            Ok(delete_res) => delete_res.deleted_count == 1,
            Err(_) => false,
        };
        match UserVocabList::remove_from_user_vocab_list(db, rt, username, &phrase) {
            Ok(_) => { },
            Err(_) => { res = false; }
        }
        return res;
    }
}

impl UserVocabList {
    pub async fn get_user_vocab_list_string(db: &Database, username: &str) -> Option<String> {
        let coll = (*db).collection(USER_VOCAB_LIST_COLL_NAME);
        let query_doc = doc! { "username": username };
        let res = match coll.find_one(query_doc, None).await {
            Ok(query_res) => {
                match query_res {
                    Some(doc) => Some(doc.get("unique_phrase_list").and_then(Bson::as_str).unwrap().to_string()),
                    None => None
                }            
            },
            Err(e) => {
                println!("Error when reading pinyin list for user {}: {:?}", username, e);
                None
            }
        };
        return res;
    }

    fn append_to_user_vocab_list(db: &Database, rt: &Handle, username: &str, new_phrase: &CnEnDictEntry) -> Result<(), Error> {
        let coll = (*db).collection(USER_VOCAB_LIST_COLL_NAME);
        let query_doc = doc! { "username": username };
        match (*rt).block_on(coll.find_one(query_doc, None)) {
            Ok(query_res) => {
                match query_res {
                    Some(doc) => {
                        // Update existing list
                        let prev_doc: UserVocabList = from_bson(Bson::Document(doc)).unwrap();
                        let mut unique_phrase_list = prev_doc.unique_phrase_list.clone();
                        // Add unique chars
                        let trad_and_simp_str = String::with_capacity(50) + &*new_phrase.trad + &*new_phrase.simp;
                        for c in (trad_and_simp_str).chars() {
                            if !unique_phrase_list.contains(c) {
                                unique_phrase_list += &c.to_string();
                                unique_phrase_list += ",";
                            }
                        }
                        // Write to db
                        prev_doc.try_update(db, rt, "unique_phrase_list", &unique_phrase_list)?;
                    }
                    None => {
                        // Create new instance with unique chars
                        let mut unique_phrase_list = String::with_capacity(50);
                        let trad_and_simp_str = String::with_capacity(50) + &*new_phrase.trad + &*new_phrase.simp;
                        for c in (trad_and_simp_str).chars() {
                            if !unique_phrase_list.contains(c) {
                                unique_phrase_list += &c.to_string();
                                unique_phrase_list += ",";
                            }
                        }
                        // Write to db
                        let username = username.to_string();
                        let new_doc = UserVocabList { username, unique_phrase_list };
                        new_doc.try_insert(db, rt)?;
                    }
                }
            },
            Err(e) => { 
                println!("Error when searching for pinyin list for user {}: {:?}", username, e);
            }
        }
        return Ok(());
    }
    
    fn remove_from_user_vocab_list(db: &Database, rt: &Handle, username: &str, phrase_to_remove: &CnEnDictEntry) -> Result<(), Error> {
        let coll = (*db).collection(USER_VOCAB_LIST_COLL_NAME);
        let query_doc = doc! { "username": username };
        match (*rt).block_on(coll.find_one(query_doc, None)) {
            Ok(query_res) => {
                match query_res {
                    Some(doc) => {
                        // Update existing list
                        let prev_doc: UserVocabList = from_bson(Bson::Document(doc)).unwrap();
                        let mut unique_phrase_list = prev_doc.unique_phrase_list.clone();
                        // Remove unique chars
                        let trad_and_simp_str = String::with_capacity(50) + &*phrase_to_remove.trad + &*phrase_to_remove.simp;
                        for c in (trad_and_simp_str).chars() {
                            if unique_phrase_list.contains(c) {
                                // remove the string from unique_phrase_list
                                let c_with_comma = format!("{},", c);
                                unique_phrase_list = unique_phrase_list.replace(&c_with_comma, "");
                            }
                        }
                        // Write to db
                        prev_doc.try_update(db, rt, "unique_phrase_list", &unique_phrase_list)?;
                    },
                    None => {}
                }
            },
            Err(e) => { 
                println!("Error when searching for pinyin list for user {}: {:?}", username, e);
            }
        }
        return Ok(());
    }
}

impl UserFeedback {
    pub fn new(feedback: String, contact: String, datetime: String) -> Self {
        let new_feedback = UserFeedback { feedback, contact, datetime };
        return new_feedback;
    }
}

/* Modules */
/// Module for converting plain-text to tokenized HTML
pub mod html_rendering {
    use super::*;

    fn is_english_phrase(s: &str) -> bool {
        // English chars use 1 byte, Chinese chars use 3 bytes
        return s.len() == s.chars().count();
    }

    fn has_chinese_punctuation(s: &str) -> bool {
        // Chinese punctuation is a Chinese char, however shouldn't be rendered as such
        const PUNCT: [char; 14] = ['（', '）', '“', '”', '、', '，', '。', '《', '》', '：', '！', '？','￥', '—'];
        let mut res = false;
        for c in s.chars() {
            if PUNCT.contains(&c) {
                res = true;
                break;
            }
        }
        return res;
    }

    fn tokenize_string(s: String) -> std::io::Result<String> {
        // Connect to tokenizer service, send and read results
        let mut stream = TcpStream::connect(format!("{}:{}", TOKENIZER_HOSTNAME, TOKENIZER_PORT))?;
        stream.write(s.as_bytes())?;
        let n_bytes = s.as_bytes().len();
        let mut tokenized_bytes = vec![0; n_bytes * 8]; // max size includes original n_bytes + at most 3*n_bytes delimiters + 4*n_bytes for pinyin
        stream.read(&mut tokenized_bytes)?;
    
        let mut res = String::from_utf8(tokenized_bytes).unwrap();
        res = res.trim_matches(char::from(0)).to_string(); // remove trailing '0' chars
        return Ok(res);
    }

    pub async fn convert_string_to_tokenized_html(db: &Database, s: &str) -> String {
        const PHRASE_DELIM: char = '$';
        const PINYIN_DELIM: char = '`';
        let tokenized_string = tokenize_string(s.to_string()).expect("Tokenizer connection error");
        let n_phrases = tokenized_string.matches(PHRASE_DELIM).count();
        let coll = (*db).collection(CEDICT_COLL_NAME);
        // Estimate pre-allocated size: max ~2100 chars per phrase (conservitively 2500), 1 usize per chars
        let mut res = String::with_capacity(n_phrases * 2500);
        for token in tokenized_string.split(PHRASE_DELIM) {
            let token_vec: Vec<&str> = token.split(PINYIN_DELIM).collect();
            let phrase = token_vec[0];
            let raw_pinyin = token_vec[1];
            let formatted_pinyin = token_vec[2];
            // Skip lookup for phrases with no Chinese chars
            if is_english_phrase(&phrase) || has_chinese_punctuation(&phrase) {
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
                continue;
            }
            // TODO: add handling for Traditional/Simplified lookup
            // For each phrase, lookup as CnEnDictPhrase (2 queries: 1 as Traditional, 1 as Simplified)
            // if none match, then generate the phrase witout the pinyin
            let mut trad_query = coll.find_one(doc! { "trad": &phrase, "raw_pinyin": raw_pinyin }, None).await.unwrap();
            let mut simp_query = coll.find_one(doc! { "simp": &phrase, "raw_pinyin": raw_pinyin }, None).await.unwrap();
            if trad_query == None && simp_query == None {
                trad_query = coll.find_one(doc! { "trad": &phrase, "formatted_pinyin": formatted_pinyin }, None).await.unwrap();
                simp_query = coll.find_one(doc! { "simp": &phrase, "formatted_pinyin": formatted_pinyin }, None).await.unwrap();
            }
            if trad_query == None && simp_query == None {
                // Append "not found" html
                let phrase_html = generate_html_for_not_found_phrase(phrase);
                res += phrase_html.as_str();
            } else {
                // Append corresponding stored html
                let mut cn_type = CnType::Traditional;
                let cedict_doc = match trad_query {
                    Some(doc) => doc,
                    None => {
                        cn_type = CnType::Simplified;
                        simp_query.unwrap()
                    },
                };
                let entry: CnEnDictEntry = from_bson(Bson::Document(cedict_doc)).unwrap();
                match cn_type {
                    CnType::Traditional => res += entry.trad_html.as_str(),
                    CnType::Simplified => res += entry.simp_html.as_str()
                }
            }
        }
        return res;
    }
    
    pub async fn render_document_table(db: &Database, username: &str) -> String {
        // get all documents for user
        let coll = (*db).collection(USER_DOC_COLL_NAME);
        let mut res = String::new();
        res += "<table class=\"table table-striped table-hover\">\n";
        res += "<tr><th>Title</th><th>Preview</th><th>Delete</th></tr>\n";
        let query_doc = doc! { "username": username };
        match coll.find(query_doc, None).await {
            Ok(mut cursor) => {
                // add each document as a <tr> item
                while let Some(item) = cursor.next().await {
                    // unwrap BSON document
                    let user_doc = item.unwrap();
                    let UserDoc { body, title, .. } = from_bson(Bson::Document(user_doc)).unwrap(); 
                    let delete_button = format!("<a href=\"/api/delete-doc/{}\">X</a>", &title);
                    let title = format!("<a href=\"/u/{}/{}\">{}</a>", &username, &title, &title);
    
                    // from body, get first n characters as content preview
                    let n = 10;
                    let mut b = [0; 3];
                    let body_chars = body.chars();
                    let mut preview_count = 0;
                    let mut content_preview = String::with_capacity((n+1)*3 as usize); // FYI: each Chinese char is 3 bytes (1 byte = 1 usize)
                    for c in body_chars.clone() {
                        if preview_count >= n {
                            break;
                        }
                        content_preview += c.encode_utf8(&mut b);
                        preview_count += 1;
                    }
                    if body_chars.count() > preview_count {
                        content_preview += "...";
                    }
                    // let interactive_content_preview = convert_string_to_tokenized_html(db, content_preview);
                    res += format!("<tr><td>{}</td><td>{}</td><td>{}</td></tr>\n", title, content_preview, delete_button).as_str();
                }
            },
            Err(e) => {
                println!("Error when searching for documents for user {}: {:?}", username, e);
            }
        }
        res += "</table>";
        return res;
    }
    
    pub async fn render_vocab_table(db: &Database, username: &str) -> String {
        let coll = (*db).collection(USER_VOCAB_COLL_NAME);
        let mut res = String::new();
        res += "<table class=\"table table-striped table-hover\">\n";
        res += format!("<tr><th>{}</th><th>{}</th><th>{}</th></tr>\n", "Term", "Saved From", "Delete").as_str();
        let query_doc = doc! { "username": username };
        match coll.find(query_doc, None).await {
            Ok(mut cursor) => {
                // add each document as a <tr> item
                while let Some(item) = cursor.next().await {
                    // unwrap BSON document
                    let user_doc = item.unwrap();
                    let UserVocab { from_doc_title, phrase, cn_type, .. } = from_bson(Bson::Document(user_doc)).unwrap();
                    let CnEnDictEntry { trad, simp, trad_html, simp_html, .. } = phrase;
                    // TODO add user settings for traditional/simplified config. For now, default to traditional
                    let hanzi = match cn_type {
                        CnType::Traditional => &trad,
                        CnType::Simplified => &simp
                    };
                    let hanzi_html = match cn_type {
                        CnType::Traditional => &trad_html,
                        CnType::Simplified => &simp_html
                    };
                    let delete_button = format!("<a href=\"/api/delete-vocab/{}\">X</a>", hanzi);
                    let row = format!("<tr><td>{}</td><td>{}</td><td>{}</td></tr>\n", hanzi_html, &from_doc_title, &delete_button);
                    res += &row;
                }
            },
            Err(e) => {
                println!("Error when searching for vocab for user {}: {:?}", username, e);
            }
        }
        res += "</table>";
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
}

/// Module for Cookie generation and handling
pub mod cookie_handling {
    use super::*;
    use rocket::http::{Cookie, Cookies, SameSite};
    use std::collections::HashMap;

    pub static JWT_NAME: &str = "duguo-代币";

    pub fn generate_http_cookie(username: String, password: String) -> Cookie<'static> {
        let jwt = match generate_jwt(username, password) {
            Ok(token) => token,
            Err(e) => {
                println!("Error when generating jwt: {:?}", e);
                String::new()
            }
        };
        let mut cookie = Cookie::new(JWT_NAME, jwt);
        cookie.set_http_only(true);
        cookie.set_same_site(SameSite::Strict);
        cookie.set_path("/");
        return cookie;
    }

    pub async fn add_user_cookie_to_context(cookies: &Cookies<'_>, db: &Database, context: &mut HashMap<&str, String>) -> bool {
        let cookie_lookup = (*cookies).get(JWT_NAME);
        let username_from_cookie = get_username_from_cookie(db, cookie_lookup).await;
        let res = match username_from_cookie {
            Some(username) => { (*context).insert("username", username); true},
            None => { false }
        };
        return res;
    }

    pub async fn get_username_from_cookie(db: &Database, cookie_lookup: Option<&Cookie<'static>>) -> Option<String> {
        let mut res = None;
        if let Some(ref login_cookie) = cookie_lookup {
            let jwt_to_check = login_cookie.value();
            res = validate_jwt_and_get_username(db, jwt_to_check).await;
        }
        return res;
    }
}