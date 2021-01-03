/// FYI: there is a private "config.rs" file where I define and implement the imports below.
/// All upper-case are static variables, and functions in config.rs are explicitly listed.
/// For security reasons, they are kept private.
mod config;
use config::{
    DB_HOSTNAME, DB_PORT, DATABASE_NAME,
    DB_USERNAME, DB_PASSWORD,
    USER_COLL_NAME, SANDBOX_COLL_NAME, TOKENIZER_PORT, TOKENIZER_HOSTNAME,
    USER_DOC_COLL_NAME, USER_VOCAB_COLL_NAME, CEDICT_COLL_NAME, 
    USER_VOCAB_LIST_COLL_NAME, USER_FEEDBACK_COLL_NAME,
    functions::{
        str_to_hashed_string, 
        generate_jwt, 
        validate_jwt_and_get_username,
    }
};
use futures::StreamExt;
use tokio::runtime::Handle;
use mongodb::{
    bson::{doc, Bson, document::Document, to_document, from_bson, to_bson},
    error::Error,
    Client, Collection, Database
};
use rocket::{
    http::{RawStr, Cookie, SameSite},
};
use std::io::prelude::*;
use std::net::TcpStream;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/* Static Vars */
// Note: other static variables used are imported from config (which is private)
pub static JWT_NAME: &str = "duguo-代币";

/* Structs */
#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    username: String,
    pw_hash: String,
    email: String,
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

#[derive(Serialize, Deserialize, Clone, Debug)]
enum CnType {
    Traditional,
    Simplified
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

/* Traits */
pub trait DatabaseItem {
    // Default
    fn as_document(&self) -> Document where Self: Serialize {
        return to_document(self).unwrap();
    }
    fn as_bson(&self) -> Bson where Self: Serialize {
        return to_bson(self).unwrap();
    }
    fn try_insert(&self, db: Database, rt: Handle) -> Result<String, Error> where Self: Serialize {
        let coll = db.collection(self.collection_name());
        let new_doc = self.as_document();
        match rt.block_on(insert_one_doc(coll, new_doc)) {
            Ok(_) => {}
            Err(e) => { return Err(e); }
        }
        return Ok(self.primary_key());
    }
    fn try_update(&self, db: Database, rt: Handle, key: &str, new_value: &str) -> Result<String, Error> where Self: Serialize {
        let coll = db.collection(self.collection_name());
        let update_doc = doc! { key: new_value };
        let update_query = doc! { "$set": update_doc };
        match rt.block_on(coll.update_one(self.as_document(), update_query, None)) {
            Ok(_) => {},
            Err(e) => { return Err(e); }
        }
        return Ok(self.primary_key());
    }

    // Requires Implementation
    fn collection_name(&self) -> &str;
    fn primary_key(&self) -> String;
}

/* Struct Functionality */
impl User {
    pub fn new(username: String, password: String, email: String) -> Self {
        let pw_hash = str_to_hashed_string(password.as_str());
        let new_user = User { username, pw_hash, email };
        return new_user;
    }
}

impl DatabaseItem for User {
    fn collection_name(&self) -> &str { return USER_COLL_NAME; }
    fn primary_key(&self) -> String {
        // TODO: technically this is a dual-key, username+email. Does this matter here?
        return self.username.clone();
    }

    // TODO: clean this up s.t. String returned is primary key (move error message logic elsewhere)
    fn try_insert(&self, db: Database, rt: Handle) -> Result<String, Error> {
        let coll = db.collection(USER_COLL_NAME);
        let username_query = check_if_username_exists(db.clone(), self.username.as_str());
        let email_query = check_coll_for_existing_key_value(coll.clone(), "email", self.email.as_str(), None);
        let is_existing_username = rt.block_on(username_query);
        let is_existing_email = rt.block_on(email_query);
        let can_register = !(is_existing_username || is_existing_email);
        let mut message = String::new();
        if can_register {
            let new_doc = self.as_document();
            match rt.block_on(insert_one_doc(coll, new_doc)) {
                Ok(_) => {
                    let success_msg = format!("Registration successful! Username: {}", self.username);
                    message.push_str(&success_msg);
                }
                Err(e) => { return Err(e); }
            }
        }
        return Ok(message);
    }
}

impl SandboxDoc {
    pub async fn new(db: Database, body: String) -> Self {
        let doc_id = Uuid::new_v4().to_string();
        let body_html = convert_string_to_tokenized_html(db.clone(), body.clone()).await;
        let new_doc = SandboxDoc { doc_id, body, body_html };
        return new_doc;
    }
}

impl DatabaseItem for SandboxDoc {
    fn collection_name(&self) -> &str { return SANDBOX_COLL_NAME; }
    fn primary_key(&self) -> String { return self.doc_id.clone(); }
}

impl UserDoc {
    pub async fn new(db: Database, username: String, desired_title: String, body: String) -> Self {
        let body_html = convert_string_to_tokenized_html(db.clone(), body.clone()).await;
        // If title is non-unique, try appending digits until match
        let coll = db.collection(USER_DOC_COLL_NAME);
        let mut title_exists = check_coll_for_existing_key_value(coll.clone(), "title", &desired_title, Some(&username)).await;
        let title = match title_exists {
            true => {
                // Try new titles until unique one found
                let mut count = 0;
                let mut new_title = String::new();
                while title_exists {
                    count += 1;
                    let appended = format!("-{}", count);
                    new_title = desired_title.clone() + appended.as_str();
                    title_exists = check_coll_for_existing_key_value(coll.clone(), "title", &new_title, Some(&username)).await;
                }
                new_title
            },
            false => { desired_title }
        };
        let new_doc = UserDoc { username, title, body, body_html };
        return new_doc;
    }

    pub async fn get_body_html_from_user_doc(db: Database, username: &str, title: &str) -> Option<String> {
        let coll = db.collection(USER_DOC_COLL_NAME);
        let query_doc = doc! { "username": username, "title": title };
        let doc_body = match coll.find_one(query_doc, None).await.unwrap() {
            Some(doc) => Some(doc.get("body_html").and_then(Bson::as_str).unwrap().to_string()),
            None => None
        };
        return doc_body;
    }

    pub async fn try_delete(db: Database, username: &str, title: &str) -> bool {
        let coll = db.collection(USER_DOC_COLL_NAME);
        let query_doc = doc! { "username": username, "title": title }; 
        let res = match coll.delete_one(query_doc, None).await {
            Ok(delete_res) => delete_res.deleted_count == 1,
            Err(_) => false,
        };
        return res;
    }
}

impl DatabaseItem for UserDoc {
    fn collection_name(&self) -> &str { return USER_DOC_COLL_NAME; }
    fn primary_key(&self) -> String { 
        // TODO: technically this is a dual key (username+title), how to handle?
        return self.username.clone(); 
    }
}

impl CnEnDictEntry {
    pub async fn new(db: Database, phrase_str: &str) -> Self {
        // Try simplified, then traditional
        let res = match CnEnDictEntry::lookup_phrase(db.clone(), "simp", phrase_str).await {
            Some(obj) => obj,
            None => {
                match CnEnDictEntry::lookup_phrase(db.clone(), "trad", phrase_str).await {
                    Some(obj) => obj,
                    None => CnEnDictEntry::generate_empty_phrase()
                }
            }
        };
        return res;
    }

    // TODO: refactor this to apply for DatabaseItem trait
    pub async fn lookup_phrase(db: Database, key: &str, value: &str) -> Option<Self> {
        let coll = db.collection(CEDICT_COLL_NAME);
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

impl DatabaseItem for CnEnDictEntry {
    fn collection_name(&self) -> &str { return CEDICT_COLL_NAME; }
    fn primary_key(&self) -> String { return self.trad.clone(); }
}

impl UserVocab {
    pub async fn new(db: Database, username: String, saved_phrase: String, from_doc_title: String) -> Self {
        // Try simplified, then traditional
        let mut cn_type = CnType::Simplified;
        let phrase: CnEnDictEntry = match CnEnDictEntry::lookup_phrase(db.clone(), "simp", &saved_phrase).await {
            Some(sp) => sp,
            None => {
                cn_type = CnType::Traditional;
                match CnEnDictEntry::lookup_phrase(db.clone(), "trad", &saved_phrase).await {
                    Some(tp) => tp,
                    None => CnEnDictEntry::generate_empty_phrase()
                }
            }
        };
        let new_vocab = UserVocab { username, from_doc_title, phrase, cn_type };
        return new_vocab;
    }

    pub fn try_delete(db: Database, rt: Handle, username: String, phrase: CnEnDictEntry) -> bool {
        let coll = db.collection(USER_VOCAB_COLL_NAME);
        let query_doc = doc! { "username": &username, "phrase.trad": &phrase.trad, "phrase.simp": &phrase.simp };
        let mut res = match rt.block_on(coll.delete_one(query_doc, None)) {
            Ok(delete_res) => delete_res.deleted_count == 1,
            Err(_) => false,
        };
        match remove_from_phrase_list_string(db.clone(), rt, &username, &phrase) {
            Ok(_) => { },
            Err(_) => { res = false; }
        }
        return res;
    }
}

impl DatabaseItem for UserVocab {
    fn try_insert(&self, db: Database, rt: Handle) -> Result<String, Error> where Self: Serialize {
        let coll = db.collection(self.collection_name());
        let new_doc = self.as_document();
        match rt.block_on(insert_one_doc(coll, new_doc)) {
            Ok(_) => {
                append_to_user_phrase_list_string(db, rt.clone(), &self.username, &self.phrase)?;
            },
            Err(e) => { return Err(e); }
        }
        return Ok(self.primary_key());
    }

    fn collection_name(&self) -> &str { return USER_VOCAB_COLL_NAME; }
    fn primary_key(&self) -> String {
        // TODO: make this phrase.trad + phrase.simp (this is unique in CEDICT), or something better...
        return self.phrase.trad.clone();
    }
}

impl DatabaseItem for UserVocabList {
    fn collection_name(&self) -> &str { return USER_VOCAB_LIST_COLL_NAME; }
    fn primary_key(&self) -> String { return self.username.clone(); }
}

impl UserFeedback {
    pub fn new(feedback: String, contact: String, datetime: String) -> Self {
        let new_feedback = UserFeedback { feedback, contact, datetime };
        return new_feedback;
    }
}

impl DatabaseItem for UserFeedback {
    fn collection_name(&self) -> &str { return USER_FEEDBACK_COLL_NAME; }
    fn primary_key(&self) -> String { return self.datetime.clone(); }
}

/* Public Functions */
pub fn connect_to_mongodb(rt: Handle) -> Result<Database, Error> {
    let uri = format!("mongodb://{}:{}@{}:{}/", DB_USERNAME, DB_PASSWORD, DB_HOSTNAME, DB_PORT);
    let client = rt.block_on(Client::with_uri_str(&uri))?;
    let db: Database = client.database(DATABASE_NAME);
    return Ok(db);
}

/// Returns String::new() if UTF-8 error is encountered
pub fn convert_rawstr_to_string(s: &RawStr) -> String {
    let mut res = match s.url_decode() {
        Ok(val) => val,
        Err(e) => {
            println!("UTF-8 Error: {:?}", e);
            String::new()
        }
    };
    // Note: can't sanitize '/' since that breaks default character encoding
    res = res.replace(&['<', '>', '(', ')', ',', '\"', ';', ':', '\''][..], "");
    return res;
}

// TODO: rewrite this as generic function as part of DatabaseItem Trait?
pub async fn get_sandbox_document(db: Database, doc_id: String) -> Option<String> {
    let coll = db.collection(SANDBOX_COLL_NAME);
    let query_doc = doc! { "doc_id": doc_id };
    let res = match coll.find_one(query_doc, None).await.unwrap() {
        Some(doc) => Some(doc.get("body").and_then(Bson::as_str).expect("No body was stored").to_string()),
        None => None
    };
    return res;
}

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

pub async fn get_username_from_cookie(db: Database, cookie_lookup: Option<&Cookie<'static>>) -> Option<String> {
    let mut res = None;
    if let Some(ref login_cookie) = cookie_lookup {
        let jwt_to_check = login_cookie.value();
        res = validate_jwt_and_get_username(db.clone(), jwt_to_check.to_string()).await;
    }
    return res;
}

pub async fn check_password(db: Database, username: String, pw_to_check: String) -> bool {
    let coll = db.collection(USER_COLL_NAME);
    let hashed_pw = str_to_hashed_string(pw_to_check.as_str());
    let query_doc = doc! { "username": username, "pw_hash": hashed_pw.clone() };
    let res = match coll.find_one(query_doc, None).await.unwrap() {
        Some(document) => {
            let saved_hash = document.get("pw_hash").and_then(Bson::as_str).expect("No password was stored");
            saved_hash == hashed_pw
        },
        None => false
    };
    return res;
}

pub async fn convert_string_to_tokenized_html(db: Database, s: String) -> String {
    let tokenized_string = tokenize_string(s).expect("Tokenizer connection error");
    let n_phrases = tokenized_string.matches(',').count();
    let coll = db.collection(CEDICT_COLL_NAME);
    // Estimate pre-allocated size: max ~2100 chars per phrase (conservitively 2500), 1 usize per chars
    let mut res = String::with_capacity(n_phrases * 2500);
    for phrase in tokenized_string.split(',') {
        // For each phrase, lookup as CnEnDictPhrase (2 queries: 1 as Traditional, 1 as Simplified)
        // if none match, then generate the phrase witout the pinyin
        let trad_query = coll.find_one(doc! { "trad": &phrase }, None).await.unwrap();
        let simp_query = coll.find_one(doc! { "simp": &phrase }, None).await.unwrap();
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

pub async fn render_document_table(db: Database, username: &str) -> String {
    // get all documents for user
    let coll = db.collection(USER_DOC_COLL_NAME);
    let mut res = String::new();
    res += "<table class=\"table table-striped table-hover\">\n";
    res += "<tr><th>Title</th><th>Preview</th><th>Delete</th></tr>\n";
    let query_doc = doc! { "username": username };
    match coll.find(query_doc, None).await {
        Ok(mut cursor) => {
            // add each document as a <tr> item
            while let Some(item) = cursor.next().await {
                // unwrap BSON document
                let user_doc = item.unwrap(); // TODO handling error case would be better (Result)
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
                // let interactive_content_preview = convert_string_to_tokenized_html(db.clone(), content_preview);
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

pub async fn render_vocab_table(db: Database, username: &str) -> String {
    let coll = db.collection(USER_VOCAB_COLL_NAME);
    let mut res = String::new();
    res += "<table class=\"table table-striped table-hover\">\n";
    res += format!("<tr><th>{}</th><th>{}</th><th>{}</th></tr>\n", "Term", "Saved From", "Delete").as_str();
    let query_doc = doc! { "username": username };
    match coll.find(query_doc, None).await {
        Ok(mut cursor) => {
            // add each document as a <tr> item
            while let Some(item) = cursor.next().await {
                // unwrap BSON document
                let user_doc = item.unwrap(); // TODO handling error case would be better (Result)
                let UserVocab { from_doc_title, phrase, cn_type, .. } = from_bson(Bson::Document(user_doc)).unwrap();
                let CnEnDictEntry { trad, simp, trad_html, simp_html, .. } = phrase;
                // TODO add user settings for traditional/simplified config. For now, default to traditional
                // TODO convert doc_title to <a href=...></a> link
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

// TODO: make this generic DatabaseItem function? Then can call User::check_if_existing_key_value("username", ...)
pub async fn check_if_username_exists(db: Database, username: &str) -> bool {
    let coll = db.collection(USER_COLL_NAME);
    let username_search = coll.find_one(doc! { "username": username }, None).await.unwrap();
    return username_search != None;
}

pub async fn get_user_vocab_list_string(db: Database, username: &str) -> Option<String> {
    let coll = db.collection(USER_VOCAB_LIST_COLL_NAME);
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

/* Private Functions */
fn append_to_user_phrase_list_string(db: Database, rt: Handle, username: &str, new_phrase: &CnEnDictEntry) -> Result<(), Error> {
    let coll = db.collection(USER_VOCAB_LIST_COLL_NAME);
    let query_doc = doc! { "username": username };
    match rt.block_on(coll.find_one(query_doc, None)) {
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
                    prev_doc.try_update(db.clone(), rt.clone(), "unique_phrase_list", &unique_phrase_list)?;
                }
                None => {
                    // Create new instance with unique chars, save to db
                    let mut unique_phrase_list = String::with_capacity(50);
                    let trad_and_simp_str = String::with_capacity(50) + &*new_phrase.trad + &*new_phrase.simp;
                    for c in (trad_and_simp_str).chars() {
                        if !unique_phrase_list.contains(c) {
                            unique_phrase_list += &c.to_string();
                            unique_phrase_list += ",";
                        }
                    }
                    let username = username.to_string();
                    let new_doc = UserVocabList { username, unique_phrase_list };
                    new_doc.try_insert(db.clone(), rt.clone())?;
                }
            }
        },
        Err(e) => { 
            println!("Error when searching for pinyin list for user {}: {:?}", username, e);
        }
    }
    return Ok(());
}

fn remove_from_phrase_list_string(db: Database, rt: Handle, username: &str, phrase_to_remove: &CnEnDictEntry) -> Result<(), Error> {
    let coll = db.collection(USER_VOCAB_LIST_COLL_NAME);
    let query_doc = doc! { "username": username };
    match rt.block_on(coll.find_one(query_doc, None)) {
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
                    prev_doc.try_update(db.clone(), rt.clone(), "unique_phrase_list", &unique_phrase_list)?;
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

async fn insert_one_doc(coll: Collection, doc: Document) -> Result<(), Error> {
    coll.delete_one(doc.clone(), None).await?; // remove once can set unique indices
    match coll.insert_one(doc.clone(), None).await {
        Ok(_) => {}
        Err(e) => {
            println!("Skipping insert for doc: {:?}\n\tGot error: {:?}", doc, e);
        }
    }
    return Ok(());
}

fn tokenize_string(s: String) -> std::io::Result<String> {
    // Connect to tokenizer service, send and read results
    let mut stream = TcpStream::connect(format!("{}:{}", TOKENIZER_HOSTNAME, TOKENIZER_PORT))?;
    stream.write(s.as_bytes())?;
    let n_bytes = s.as_bytes().len();
    let mut tokenized_bytes = vec![0; n_bytes * 2]; // max size includes original 'n_bytes' + at most 'n_bytes' commas
    stream.read(&mut tokenized_bytes)?;

    let mut res = String::from_utf8(tokenized_bytes).unwrap();
    res = res.trim_matches(char::from(0)).to_string(); // remove training '0' chars
    return Ok(res);
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

// TODO: make this generic DatabaseItem function? Then can call Struct::check_if_existing_key_value("username", ...)
async fn check_coll_for_existing_key_value(coll: Collection, key: &str, value: &str, username: Option<&str>) -> bool {
    let query = match username {
        Some(u) => coll.find_one(doc! { "username": u, key: value }, None).await.unwrap(),
        None => coll.find_one(doc! { key: value }, None).await.unwrap()
    };
    return query != None;
}