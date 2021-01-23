/// Helper Functions
/// 
/// Structure:
/// 
/// duguo (lib.rs)
/// ├── connect_to_mongodb: Fn
/// ├── connect_to_redis: Fn
/// ├── convert_rawstr_to_string: Fn
/// |
/// ├── {CacheItem, DatabaseItem}: Traits
/// |   └── Impl for all Traits
/// ├── {CnType, CnPhonetics}: Enums
/// |   └── Impl for all Enums
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
use mongodb::{
    bson::{doc, Bson, document::Document, to_document, from_bson, to_bson},
    error::Error,
    sync::Database
};
use rocket::http::RawStr;
use scraper::{Html, Selector};
use std::{
    fmt,
    collections::HashMap,
    io::prelude::*,
    net::TcpStream,
};
use chrono::Utc;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use redis::{
    AsyncCommands,
    aio::Connection
};


/* Public Functions */
/// Connects to mongoDB (locally: Docker Container, in production: mongoDB Atlas). Connection is handled in main.rs
pub fn connect_to_mongodb() -> Result<Database, Error> {
    let client = mongodb::sync::Client::with_uri_str(DB_URI)?;
    let db: Database = client.database(DB_NAME);
    return Ok(db);
}

/// Uses URI to connect to Redis (Docker Container). Connections are handled within lib.rs
async fn connect_to_redis() -> Result<Connection, Box<dyn std::error::Error>> {
    let client = redis::Client::open(REDIS_URI)?;
    let conn = client.get_async_connection().await?;
    return Ok(conn);
}

/// Sanitizes user input
pub fn convert_rawstr_to_string(s: &RawStr) -> String {
    let mut res = s.percent_decode_lossy().to_string(); // � for invalid UTF-8
    // Note: can't sanitize '/' since that breaks urls
    res = res.replace(&['<', '>', '(', ')', '!', '\"', '\'', '\\', ';', '{', '}', ':', '*'][..], "");
    return res;
}

/* Enums */
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum CnType {
    Traditional,
    Simplified
}

impl CnType {
    pub fn as_str(&self) -> &str {
        return match *self {
            CnType::Traditional => "Traditional",
            CnType::Simplified => "Simplified"
        };
    }

    fn from_str(s: &str) -> Self {
        return match s {
            "Traditional" => CnType::Traditional,
            "traditional" => CnType::Traditional,
            "trad" => CnType::Traditional,
            "Simplified" => CnType::Simplified,
            "simplified" => CnType::Simplified,
            "simp" => CnType::Simplified,
            _ => CnType::Simplified // Default to simplified
        }
    }
}

/// Implements to_string()
impl fmt::Display for CnType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return write!(f, "{}", self.as_str());
    }
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum CnPhonetics {
    Pinyin,
    Zhuyin
}

impl CnPhonetics {
    pub fn as_str(&self) -> &str {
        return match *self {
            CnPhonetics::Pinyin => "Pinyin",
            CnPhonetics::Zhuyin => "Zhuyin"
        };
    }

    fn from_str(s: &str) -> Self {
        return match s {
            "Pinyin" => CnPhonetics::Pinyin,
            "pinyin" => CnPhonetics::Pinyin,
            "Zhuyin" => CnPhonetics::Zhuyin,
            "zhuyin" => CnPhonetics::Zhuyin,
            "Bopomofo" => CnPhonetics::Zhuyin,
            "bopomofo" => CnPhonetics::Zhuyin,
            _ => CnPhonetics::Pinyin // Default to pinyin
        }
    }
}

/// Implements to_string()
impl fmt::Display for CnPhonetics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return write!(f, "{}", self.as_str());
    }
}


/* Structs */
#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    username: String,
    pw_hash: String,
    email: String,
    cn_type: CnType,
    cn_phonetics: CnPhonetics,
    created_on: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SandboxDoc {
    doc_id: String,
    body: String,
    body_html: String,
    // If none, String::new()
    from_url: String,
    cn_type: CnType,
    cn_phonetics: CnPhonetics,
    created_on: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserDoc {
    username: String,
    title: String,
    body: String,
    body_html: String,
    // If none, String::new()
    from_url: String, 
    cn_type: CnType,
    cn_phonetics: CnPhonetics,
    created_on: String
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CnEnDictEntry {
    uid: String,
    trad: String,
    simp: String,
    raw_pinyin: String,
    formatted_pinyin: String,
    trad_html: String,
    simp_html: String,
    def: String,
    zhuyin: String,
    trad_zhuyin_html: String,
    simp_zhuyin_html: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserVocab {
    username: String,
    from_doc_title: String,
    cn_type: CnType,
    cn_phonetics: CnPhonetics,
    phrase: String,
    def: String, 
    /// If pinyin, formatted_pinyin
    phrase_phonetics: String, 
    phrase_html: String,
    created_on: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserVocabList {
    username: String,
    /// Comma-delimited String
    unique_phrase_list: String, 
    cn_type: CnType
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserFeedback {
    feedback: String,
    /// If none, String::new()
    contact: String, 
    created_on: String
}


/* Traits */
pub trait CacheItem {
    /// For CnEnDictEntry, ordered_items are: vec![simp, raw_pinyin]
    fn generate_uid(ordered_items: Vec<String>) -> String {
        let mut res = String::with_capacity(100); // 80 chars is upper-bound from largest CEDICT entry
        for item in ordered_items {
            res += &item.replace(" ", "");
        }
        return res;
    }
}

impl CacheItem for CnEnDictEntry { }

pub trait DatabaseItem {
    // Implemented Defaults
    fn as_document(&self) -> Document where Self: Serialize {
        return to_document(self).unwrap();
    }
    fn as_bson(&self) -> Bson where Self: Serialize {
        return to_bson(self).unwrap();
    }
    fn try_insert(&self, db: &Database) -> Result<String, Error> where Self: Serialize {
        let coll = (*db).collection(self.collection_name());
        let new_doc = self.as_document();
        match coll.insert_one(new_doc, None) {
            Ok(_) => {}
            Err(e) => { return Err(e); }
        }
        return Ok(self.primary_key().to_string());
    }
    fn try_update(&self, db: &Database, key: &str, new_value: &str) -> Result<String, Error> where Self: Serialize {
        let coll = (*db).collection(self.collection_name());
        let update_doc = doc! { key: new_value };
        let update_query = doc! { "$set": update_doc };
        match coll.update_one(self.as_document(), update_query, None) {
            Ok(_) => {},
            Err(e) => { return Err(e); }
        }
        return Ok(self.primary_key().to_string());
    }

    // Requires Implementation
    fn collection_name(&self) -> &str;
    fn primary_key(&self) -> &str;
}

/* Trait Implementation */
impl DatabaseItem for User {
    fn try_insert(&self, db: &Database) -> Result<String, Error> {
        let coll = (*db).collection(USER_COLL_NAME);
        let can_register = User::check_if_username_and_email_are_available(db, &self.username, &self.email);
        if can_register {
            let new_doc = self.as_document();
            match coll.insert_one(new_doc, None) {
                Ok(_) => { },
                Err(e) => { return Err(e); }
            }
        }
        return Ok(self.primary_key().to_string());
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

impl DatabaseItem for UserVocab {
    fn try_insert(&self, db: &Database) -> Result<String, Error> where Self: Serialize {
        let coll = (*db).collection(self.collection_name());
        let new_doc = self.as_document();
        match coll.insert_one(new_doc, None) {
            Ok(_) => { UserVocabList::append_to_user_vocab_list(db, &self.username, &self.phrase, self.cn_type.as_str())?; },
            Err(e) => { return Err(e); }
        }
        return Ok(self.primary_key().to_string());
    }

    fn collection_name(&self) -> &str { return USER_VOCAB_COLL_NAME; }
    fn primary_key(&self) -> &str { return &self.phrase_html; }
}

impl DatabaseItem for UserVocabList {
    fn collection_name(&self) -> &str { return USER_VOCAB_LIST_COLL_NAME; }
    /// NOTE: this is not necessarily unique per user, a unique primary key is username + cn_type
    fn primary_key(&self) -> &str { return &self.username; } 
}

impl DatabaseItem for UserFeedback {
    fn collection_name(&self) -> &str { return USER_FEEDBACK_COLL_NAME; }
    fn primary_key(&self) -> &str { return &self.created_on; }
}

/* Struct Implementation */
impl User {
    fn default_settings() -> (CnType, CnPhonetics) {
        return (CnType::Simplified, CnPhonetics::Pinyin);
    }

    pub fn new(username: String, password: String, email: String) -> Self {
        let pw_hash = str_to_hashed_string(&password);
        let (cn_type, cn_phonetics) = User::default_settings();
        let created_on = Utc::now().to_string();
        let new_user = User { username, pw_hash, email, cn_type, cn_phonetics, created_on };
        return new_user;
    }

    pub fn check_if_username_exists(db: &Database, username: &str) -> bool {
        let coll = (*db).collection(USER_COLL_NAME);
        return (coll.find_one(doc! {"username": username }, None).unwrap()) != None;
    }

    pub fn update_user_settings(db: &Database, username: &str, cn_type: Option<CnType>, cn_phonetics: Option<CnPhonetics>) -> Result<(), Error> {
        let user = User::from_username(db, username).unwrap();
        if let Some(new_type) = cn_type {
            user.try_update(db, "cn_type", new_type.as_str())?;
        }
        if let Some(new_phonetics) = cn_phonetics {
            user.try_update(db, "cn_phonetics", new_phonetics.as_str())?;
        }
        return Ok(());
    }

    pub fn get_user_settings(db: &Database, username: &str) -> (CnType, CnPhonetics) {
        let coll = (*db).collection(USER_COLL_NAME);
        let res_tup = match coll.find_one(doc! {"username": username }, None).unwrap() {
            Some(user_doc) => {
                let User { cn_type, cn_phonetics, ..} = from_bson(Bson::Document(user_doc)).unwrap();
                (cn_type, cn_phonetics)
            },
            None => User::default_settings()
        };
        return res_tup;
    }

    pub fn check_password(db: &Database, username: &str, pw_to_check: &str) -> bool {
        let coll = (*db).collection(USER_COLL_NAME);
        let hashed_pw = str_to_hashed_string(pw_to_check);
        let query_doc = doc! { "username": username, "pw_hash": &hashed_pw };
        let res = match coll.find_one(query_doc, None).unwrap() {
            Some(document) => {
                let saved_hash = document.get("pw_hash").and_then(Bson::as_str).expect("No password was stored");
                saved_hash == &hashed_pw
            },
            None => false
        };
        return res;
    }

    fn from_username(db: &Database, username: &str) -> Option<Self> {
        let coll = (*db).collection(USER_COLL_NAME);
        let query_res = coll.find_one(doc! {"username": username}, None).unwrap();
        let res: Option<Self> = match query_res {
            Some(doc) => Some(from_bson(Bson::Document(doc)).unwrap()),
            None => None,
        };
        return res;
    }

    fn check_if_username_and_email_are_available(db: &Database, username: &str, email: &str) -> bool {
        let coll = (*db).collection(USER_COLL_NAME);
        let username_query = coll.find_one(doc! {"username": username }, None).unwrap();
        let email_query = coll.find_one(doc! {"email": email}, None).unwrap();
        return (username_query == None) && (email_query == None);
    }
}

impl SandboxDoc {
    pub async fn new(body: String, cn_type: String, cn_phonetics: String, url: Option<String>) -> Self {
        let doc_id = Uuid::new_v4().to_string();
        let cn_type = CnType::from_str(&cn_type);
        let cn_phonetics = CnPhonetics::from_str(&cn_phonetics);
        let body_html = html_rendering::convert_string_to_tokenized_html(&body, &cn_type, &cn_phonetics).await;
        let from_url = match url {
            Some(url) => url,
            None => String::new()
        };
        let created_on = Utc::now().to_string();
        let new_doc = SandboxDoc { doc_id, body, body_html, from_url, cn_type, cn_phonetics, created_on };
        return new_doc;
    }

    pub async fn from_url(url: String, cn_type: String, cn_phonetics: String) -> Self {
        // make request
        let resp = reqwest::get(&url).await.unwrap()
            .text().await.unwrap();
        let html = Html::parse_document(&resp);
        // get body from all headers, paragraphs in-order
        let body_selector = Selector::parse("body h1,h2,h3,h4,h5,h6,p").unwrap();
        let mut body_text = String::with_capacity(resp.len());
        for item in html.select(&body_selector) {
            body_text += &item.text().collect::<String>();
        }
        return SandboxDoc::new(body_text, cn_type, cn_phonetics, Some(url)).await;
    }

    pub fn get_doc_html_and_phonetics_from_id(db: &Database, doc_id: String) -> Option<(String, String)> {
        let coll = (*db).collection(SANDBOX_COLL_NAME);
        let query_doc = doc! { "doc_id": doc_id };
        let mut doc_html = String::new();
        let mut cn_phonetics = String::new();
        let res = match coll.find_one(query_doc, None).unwrap() {
            Some(doc) => {
                doc_html += doc.get("body_html").and_then(Bson::as_str).expect("No body_html was stored");
                cn_phonetics += doc.get("cn_phonetics").and_then(Bson::as_str).expect("No phonetic info was stored");
                Some((doc_html, cn_phonetics))       
            },
            None => None
        };
        return res;
    }
}

impl UserDoc {
    pub async fn new(db: &Database, username: String, desired_title: String, body: String, url: Option<String>) -> Self {
        let (cn_type, cn_phonetics) = User::get_user_settings(db, &username);
        let body_html = html_rendering::convert_string_to_tokenized_html(&body, &cn_type, &cn_phonetics).await;
        let desired_title = desired_title.replace(" ", "");
        // If title is non-unique, try appending digits until match
        let coll = (*db).collection(USER_DOC_COLL_NAME);
        let mut title_exists = (coll.find_one(doc! {"username": &username, "title": &desired_title, "cn_type": cn_type.as_str(), "cn_phonetics": cn_phonetics.as_str()}, None).unwrap()) != None;
        let title = match title_exists {
            true => {
                // Try new titles until unique one found
                let mut count = 0;
                let mut new_title = String::new();
                while title_exists {
                    count += 1;
                    let appended = format!("-{}", count);
                    new_title = desired_title.clone() + appended.as_str(); // need .clone() here because of loop
                    title_exists = (coll.find_one(doc! {"username": &username, "title": &new_title, "cn_type": cn_type.as_str(), "cn_phonetics": cn_phonetics.as_str()}, None).unwrap()) != None;
                }
                new_title
            },
            false => { desired_title }
        };
        let from_url = match url {
            Some(url) => url,
            None => String::new()
        };
        let created_on = Utc::now().to_string();
        let new_doc = UserDoc { username, title, body, body_html, from_url, cn_type, cn_phonetics, created_on };
        return new_doc;
    }

    pub async fn from_url(db: &Database, username: String, url: String) -> Self {
        // make request
        let resp = reqwest::get(&url).await.unwrap()
            .text().await.unwrap();
        let html = Html::parse_document(&resp);
        // get title
        let title_selector = Selector::parse("title").unwrap();
        let title_text: String = html.select(&title_selector)
            .next().unwrap()
            .text().collect();
        // get body from all headers, paragraphs in-order
        let body_selector = Selector::parse("body h1,h2,h3,h4,h5,h6,p").unwrap();
        let mut body_text = String::with_capacity(resp.len());
        for item in  html.select(&body_selector) {
            body_text += &item.text().collect::<String>();
        }
        return UserDoc::new(db, username, title_text, body_text, Some(url)).await;
    }

    pub fn get_body_html_from_user_doc(db: &Database, username: &str, title: &str) -> Option<String> {
        let (cn_type, cn_phonetics) = User::get_user_settings(&db, &username);
        let coll = (*db).collection(USER_DOC_COLL_NAME);
        let query_doc = doc! { "username": username, "title": title, "cn_type": cn_type.as_str(), "cn_phonetics": cn_phonetics.as_str() };
        let doc_body = match coll.find_one(query_doc, None).unwrap() {
            Some(doc) => Some(doc.get("body_html").and_then(Bson::as_str).unwrap().to_string()),
            None => None
        };
        return doc_body;
    }

    pub fn try_delete(db: &Database, username: &str, title: &str) -> bool {
        let coll = (*db).collection(USER_DOC_COLL_NAME);
        let query_doc = doc! { "username": username, "title": title }; 
        let res = match coll.delete_one(query_doc, None) {
            Ok(delete_res) => delete_res.deleted_count == 1,
            Err(_) => false,
        };
        return res;
    }
}

impl CnEnDictEntry {
    pub async fn from_uid(conn: &mut Connection, uid: String) -> Self {
        let query_map = (*conn).hgetall::<&str, HashMap<String, String>>(&uid).await.unwrap();
        let res = match query_map.len() {
            0 => CnEnDictEntry::generate_lookup_failed_entry(&uid),
            _ => CnEnDictEntry {
                    uid,
                    trad : query_map.get("trad").unwrap().to_owned(),
                    simp: query_map.get("simp").unwrap().to_owned(),
                    raw_pinyin: query_map.get("raw_pinyin").unwrap().to_owned(),
                    formatted_pinyin: query_map.get("formatted_pinyin").unwrap().to_owned(),
                    trad_html: query_map.get("trad_html").unwrap().to_owned(),
                    simp_html: query_map.get("simp_html").unwrap().to_owned(),
                    def: query_map.get("def").unwrap().to_owned(),
                    zhuyin: query_map.get("zhuyin").unwrap().to_owned(),
                    trad_zhuyin_html: query_map.get("trad_zhuyin_html").unwrap().to_owned(),
                    simp_zhuyin_html: query_map.get("simp_zhuyin_html").unwrap().to_owned(),
                }
        };
        return res;
    }

    pub async fn from_tokenizer_components(conn: &mut Connection, simp: &str, raw_pinyin: &str) -> Self {
        let uid = CnEnDictEntry::generate_uid(vec![simp.to_string(), raw_pinyin.to_string()]);
        return CnEnDictEntry::from_uid(conn, uid).await;
    }

    pub fn lookup_failed(&self) -> bool {
        return self.trad_html == "";
    }

    fn get_vocab_data(&self, cn_type: &CnType, cn_phonetics: &CnPhonetics) -> (String, String, String, String) {
        // Order: (phrase, defn, phrase_phonetics, phrase_html)
        let defn = &self.def;
        let (phrase, phrase_phonetics, phrase_html) = match (cn_type, cn_phonetics) {
            (CnType::Traditional, CnPhonetics::Pinyin) => (&self.trad, &self.formatted_pinyin, &self.trad_html),
            (CnType::Traditional, CnPhonetics::Zhuyin) => (&self.trad, &self.zhuyin, &self.trad_zhuyin_html),
            (CnType::Simplified, CnPhonetics::Pinyin) => (&self.simp, &self.formatted_pinyin, &self.simp_html),
            (CnType::Simplified, CnPhonetics::Zhuyin) => (&self.simp, &self.zhuyin, &self.simp_zhuyin_html)
        };
        return (phrase.to_string(), defn.to_string(), phrase_phonetics.to_string(), phrase_html.to_string());
    }

    fn generate_lookup_failed_entry(uid: &str) -> Self {
        const LOOKUP_ERROR_MSG: &str = "N/A - Not found in database";
        let res = CnEnDictEntry {
            uid: String::from(uid),
            def: String::from(LOOKUP_ERROR_MSG),
            ..Default::default()
        }; 
        return res;
    }
}

impl UserVocab {
    pub async fn new(db: &Database, username: String, saved_uid: String, from_doc_title: String) -> Self {
        // For lookup, try user-specified first
        let mut conn = connect_to_redis().await.unwrap();
        let (cn_type, cn_phonetics) = User::get_user_settings(db, &username);
        let entry = CnEnDictEntry::from_uid(&mut conn, saved_uid).await;
        let created_on = Utc::now().to_string();
        // extract relevant info from phrase
        let (phrase, def, phrase_phonetics, phrase_html) = entry.get_vocab_data(&cn_type, &cn_phonetics);
        let new_vocab = UserVocab { 
            username, from_doc_title, def,
            phrase, phrase_phonetics, phrase_html,
            cn_type, cn_phonetics, created_on
        };
        return new_vocab;
    }

    pub fn try_delete(db: &Database, username: &str, phrase: &str, cn_type: &CnType) -> bool {
        let coll = (*db).collection(USER_VOCAB_COLL_NAME);
        let query_doc = doc! { "username": username, "phrase": phrase, "cn_type": cn_type.as_str() };
        let mut res = match coll.delete_one(query_doc, None) {
            Ok(delete_res) => delete_res.deleted_count == 1,
            Err(_) => false,
        };
        match UserVocabList::remove_from_user_vocab_list(db, username, phrase, cn_type) {
            Ok(_) => { },
            Err(_) => { res = false; }
        }
        return res;
    }
}

impl UserVocabList {
    pub fn get_user_vocab_list_string(db: &Database, username: &str) -> Option<String> {
        let (cn_type, _) = User::get_user_settings(db, username);
        let coll = (*db).collection(USER_VOCAB_LIST_COLL_NAME);
        let query_doc = doc! { "username": username, "cn_type": cn_type.as_str() };
        let res = match coll.find_one(query_doc, None) {
            Ok(query_res) => {
                match query_res {
                    Some(doc) => Some(doc.get("unique_phrase_list").and_then(Bson::as_str).unwrap().to_string()),
                    None => None
                }            
            },
            Err(e) => {
                eprintln!("Error when reading pinyin list for user {}: {:?}", username, e);
                None
            }
        };
        return res;
    }

    fn append_to_user_vocab_list(db: &Database, username: &str, new_phrase: &str, cn_type_str: &str) -> Result<(), Error> {
        let coll = (*db).collection(USER_VOCAB_LIST_COLL_NAME);
        let query_doc = doc! { "username": username, "cn_type": cn_type_str };
        match coll.find_one(query_doc, None) {
            Ok(query_res) => {
                match query_res {
                    Some(doc) => {
                        // Update existing list
                        let prev_doc: UserVocabList = from_bson(Bson::Document(doc)).unwrap();
                        let mut unique_phrase_list = prev_doc.unique_phrase_list.clone();
                        // Add unique chars
                        let phrase_string = String::from(new_phrase);
                        for c in (phrase_string).chars() {
                            if !unique_phrase_list.contains(c) {
                                unique_phrase_list += &c.to_string();
                                unique_phrase_list += ",";
                            }
                        }
                        // Write to db
                        prev_doc.try_update(db, "unique_phrase_list", &unique_phrase_list)?;
                    }
                    None => {
                        // Create new instance with unique chars
                        let mut unique_phrase_list = String::with_capacity(50);
                        let phrase_string = String::from(new_phrase);
                        for c in (phrase_string).chars() {
                            if !unique_phrase_list.contains(c) {
                                unique_phrase_list += &c.to_string();
                                unique_phrase_list += ",";
                            }
                        }
                        // Write to db
                        let username = username.to_string();
                        let cn_type = CnType::from_str(cn_type_str);
                        let new_doc = UserVocabList { username, unique_phrase_list, cn_type };
                        new_doc.try_insert(db)?;
                    }
                }
            },
            Err(e) => { eprintln!("Error when searching for pinyin list for user {}: {:?}", username, e); }
        }
        return Ok(());
    }
    
    fn remove_from_user_vocab_list(db: &Database, username: &str, phrase_to_remove: &str, cn_type: &CnType) -> Result<(), Error> {
        let coll = (*db).collection(USER_VOCAB_LIST_COLL_NAME);
        let query_doc = doc! { "username": username, "cn_type": cn_type.as_str() };        
        match coll.find_one(query_doc, None) {
            Ok(query_res) => {
                match query_res {
                    Some(doc) => {
                        // Update existing list
                        let prev_doc: UserVocabList = from_bson(Bson::Document(doc)).unwrap();
                        let mut unique_phrase_list = prev_doc.unique_phrase_list.clone();
                        // Remove unique chars
                        let phrase_string = String::from(phrase_to_remove);
                        for c in (phrase_string).chars() {
                            if unique_phrase_list.contains(c) {
                                // remove the string from unique_phrase_list
                                let c_with_comma = format!("{},", c);
                                unique_phrase_list = unique_phrase_list.replace(&c_with_comma, "");
                            }
                        }
                        // Write to db
                        prev_doc.try_update(db, "unique_phrase_list", &unique_phrase_list)?;
                    },
                    None => {}
                }
            },
            Err(e) => { eprintln!("Error when searching for pinyin list for user {}: {:?}", username, e); }
        }
        return Ok(());
    }
}

impl UserFeedback {
    pub fn new(feedback: String, contact: String) -> Self {
        let created_on = Utc::now().to_string();
        let new_feedback = UserFeedback { feedback, contact, created_on };
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
        res += "<thead class=\"table-light\">\n<tr><th>Title</th><th>Source</th><th>Created On (UTC)</th><th>Delete</th></tr>\n</thead>\n";
        let query_doc = doc! { "username": username, "cn_type": cn_type.as_str(), "cn_phonetics": cn_phonetics.as_str() };
        match coll.find(query_doc, None) {
            Ok(cursor) => {
                // add each document as a <tr> item
                res += "<tbody>\n";
                for item in cursor {
                    // unwrap BSON document
                    let user_doc = item.unwrap();
                    let UserDoc { title, created_on, from_url, .. } = from_bson(Bson::Document(user_doc)).unwrap(); 
                    let delete_button = format!("<a href=\"/api/delete-doc/{}\"><img src={}></img></a>", &title, TRASH_ICON);
                    let title = format!("<a href=\"/u/{}/{}\">{}</a>", &username, &title, &title);
                    let source = match from_url.as_str() {
                        "" => String::from("n/a"),
                        _ => format!("<a href=\"{}\">Link</a>", from_url)
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
        res += "<thead class=\"table-light\">\n<tr><th>Phrase</th><th>Saved From (plaintext)</th><th>Saved On (UTC)</th><th>Delete</th></tr>\n</thead>\n";
        let query_doc = doc! { "username": username, "cn_type": cn_type.as_str(), "cn_phonetics": cn_phonetics.as_str() };
        match coll.find(query_doc, None) {
            Ok(cursor) => {
                res += "<tbody>\n";
                // add each document as a <tr> item
                for item in cursor {
                    // unwrap BSON document
                    let user_doc = item.unwrap();
                    let UserVocab { from_doc_title, phrase, phrase_html, created_on, .. } = from_bson(Bson::Document(user_doc)).unwrap();
                    let delete_button = format!("<a href=\"/api/delete-vocab/{}\"><img src={}></img></a>", phrase, TRASH_ICON);
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

    pub fn add_user_cookie_to_context(cookies: &Cookies<'_>, db: &Database, context: &mut HashMap<&str, String>) -> bool {
        let cookie_lookup = (*cookies).get(JWT_NAME);
        let username_from_cookie = get_username_from_cookie(db, cookie_lookup);
        let res = match username_from_cookie {
            Some(username) => { 
                (*context).insert("username", username);
                true
            },
            None => false
        };
        return res;
    }

    pub fn get_username_from_cookie(db: &Database, cookie_lookup: Option<&Cookie<'static>>) -> Option<String> {
        let mut res = None;
        if let Some(ref login_cookie) = cookie_lookup {
            let jwt_to_check = login_cookie.value();
            res = validate_jwt_and_get_username(db, jwt_to_check);
        }
        return res;
    }
}