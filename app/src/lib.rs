/// FYI: there is a private "config.rs" file where I define and implement the imports below.
/// All upper-case are static variables, and functions are also explicitly listed.
/// For security reasons, they are kept private.
mod config;
use config::{
    DB_HOSTNAME, DB_PORT, DATABASE_NAME,
    USER_COLL_NAME, SANDBOX_COLL_NAME, TOKENIZER_PORT,
    USER_DOC_COLL_NAME, USER_VOCAB_COLL_NAME, CEDICT_COLL_NAME,
    functions::{
        str_to_hashed_string, 
        generate_jwt, 
        validate_jwt_and_get_username,
    }
};
use mongodb::{
    bson::{doc, Bson, document::Document, to_document, from_bson},
    options::{ClientOptions, StreamAddress},
    sync::{Client, Collection, Database},
    error::Error,
};
use rocket::{
    http::{RawStr, Cookie},
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
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserDoc {
    username: String,
    title: String,
    body: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct CnEnDictEntry {
    trad: String,
    simp: String,
    pinyin_raw: String,
    pinyin_formatted: String,
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
    phrase_string: String,
    cn_type: CnType,
}

#[derive(Debug)]
pub struct HtmlString(pub String);

/* Traits */
pub trait DatabaseItem {
    // Default
    fn as_document(&self) -> Document where Self: Serialize {
        return to_document(self).unwrap();
    }
    fn is_saved_to_db(&self, db: Database) -> bool where Self: Serialize {
        let query_doc = self.as_document();
        let coll = db.collection(self.collection_name());
        let res = match coll.find_one(query_doc, None).unwrap() {
            Some(_) => true,
            None => false
        };
        return res;
    }
    fn try_insert(&self, db: Database) -> Result<String, Error> where Self: Serialize {
        let coll = db.collection(self.collection_name());
        let new_doc = self.as_document();
        match insert_one_doc(coll, new_doc) {
            Ok(_) => {}
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
    fn try_insert(&self, db: Database) -> Result<String, Error> {
        let is_existing_username = check_if_username_exists(db.clone(), self.username.as_str());
        let is_existing_email = check_if_email_exists(db.clone(), self.email.as_str());
        let can_register = !(is_existing_username || is_existing_email);
        let mut message = String::new();
        if can_register {
            let user_coll = db.collection(USER_COLL_NAME);
            let new_doc = self.as_document();
            match insert_one_doc(user_coll, new_doc) {
                Ok(_) => {
                    let success_msg = format!("Registration successful! Username: {}", self.username);
                    message.push_str(&success_msg);
                }
                Err(e) => {
                    let error_msg = format!("Error when pushing to database, log: {:?}", e);
                    message.push_str(&error_msg);
                }
            }
        } else {
            if is_existing_username {
                let user_taken_msg = format!("Username {} is already in-use. ", self.username);
                message.push_str(&user_taken_msg);
            }
            if is_existing_email {
                let email_taken_msg = format!("Email {} is already in-use. ", self.email);
                message.push_str(&email_taken_msg);
            }
        }
        return Ok(message);
    }
}

impl SandboxDoc {
    pub fn new(body: String) -> Self {
        let doc_id = Uuid::new_v4().to_string();
        let new_doc = SandboxDoc { doc_id, body };
        return new_doc;
    }
}

impl DatabaseItem for SandboxDoc {
    fn collection_name(&self) -> &str { return SANDBOX_COLL_NAME; }
    fn primary_key(&self) -> String { return self.doc_id.clone(); }
}

impl UserDoc {
    pub fn new(username: String, title: String, body: String) -> Self {
        // TODO: consider using Uuid to for consistency and unique primary key?
        let new_doc = UserDoc { username, title, body };
        return new_doc;
    }

    pub fn get_body_from_user_doc(db: Database, username: &str, title: &str) -> Option<String> {
        let coll = db.collection(USER_DOC_COLL_NAME);
        let query_doc = doc! { "username": username, "title": title };
        let doc_body = match coll.find_one(query_doc, None).unwrap() {
            Some(doc) => Some(doc.get("body").and_then(Bson::as_str).unwrap().to_string()),
            None => None
        };
        return doc_body;
    }

    pub fn try_delete(db: Database, username: &str, title: &str) -> bool {
        let coll = db.collection(USER_DOC_COLL_NAME);
        let query_doc = doc! { "username": username, "title": title }; 
        let res = match coll.delete_one(query_doc, None) {
            Ok(delete_res) => delete_res.deleted_count == 1,
            Err(_) => false,
        };
        return res;
    }
    
    pub fn update_title(&mut self, db: Database, new_title: String) -> bool {
        let coll = db.collection(self.collection_name());
        let title_exists = check_coll_for_existing_key_value(coll, "title", &new_title);
        let res = match title_exists {
            true => false,
            false => { 
                self.title = new_title;
                true
            }
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
    fn try_insert(&self, db: Database) -> Result<String, Error> where Self: Serialize {
        let coll = db.collection(self.collection_name());
        let title_exists = check_coll_for_existing_key_value(coll.clone(), "title", &self.title);
        let new_doc = match title_exists {
            true => {
                let mut dup_doc = self.clone();
                dup_doc.title = self.title.clone() + "-duplicate";
                dup_doc.as_document()
            },
            false => { self.as_document() }
        };
        match insert_one_doc(coll, new_doc) {
            Ok(_) => {}
            Err(e) => { return Err(e); }
        }
        return Ok(self.primary_key());
    }
}

impl CnEnDictEntry {
    pub fn as_html(&self, cn_type: CnType) -> HtmlString {
        // TODO: add <span> with bootstrap popover tags
        let mut res = String::new();
        res += "<table style=\"display: inline-table;\">";
        let mut pinyin_td = String::new();
        for py in self.pinyin_formatted.split(" ") {
            pinyin_td += format!("<td>{}</td>", py).as_str();
        }
        res += format!("<tr>{}</tr>", pinyin_td).as_str();
        let mut phrase_td = String::new();
        let phrase = match cn_type {
            CnType::Traditional => &self.trad,
            CnType::Simplified => &self.simp,
        };
        for c in phrase.chars() {
            phrase_td += format!("<td>{}</td>", c).as_str();
        }
        res += format!("<tr>{}</tr>", phrase_td).as_str();
        res += "</table>";
        return HtmlString(res);
    }

    pub fn lookup_phrase(db: Database, phrase: String, cn_type: CnType) -> Option<Self> {
        let coll = db.collection(CEDICT_COLL_NAME);
        // Try traditional first, then simplified
        let query_doc = match cn_type {
            CnType::Traditional => doc! { "trad": &phrase },
            CnType::Simplified => doc! { "simp": &phrase }
        };
        let phrase: Option<Self> = match coll.find_one(query_doc, None).unwrap() {
            Some(doc) => Some(from_bson(Bson::Document(doc)).unwrap()),
            None => None,
        };
        return phrase;
    }

    pub fn generate_empty_phrase() -> Self {
        const LOOKUP_ERROR_MSG: &str = "N/A - Not found in CEDICT";
        let res = CnEnDictEntry {
            trad: String::from(LOOKUP_ERROR_MSG),
            simp: String::from(LOOKUP_ERROR_MSG),
            pinyin_raw: String::from(LOOKUP_ERROR_MSG),
            pinyin_formatted: String::from(LOOKUP_ERROR_MSG),
            def: String::from(LOOKUP_ERROR_MSG)
        };
        return res;
    }
}

impl DatabaseItem for CnEnDictEntry {
    fn collection_name(&self) -> &str { return CEDICT_COLL_NAME; }
    fn primary_key(&self) -> String { return self.trad.clone(); }
}

impl UserVocab {
    pub fn new(db: Database, username: String, phrase: String, from_doc_title: String) -> Self {
        // Lookup CEDICT entry in Database
        // TODO: parse-out contextual surrounding from_doc_title
        // TODO: lookup CEDICT to create the phrase

        // Try traditional first, then simplified
        let mut cn_type = CnType::Traditional;
        let phrase: CnEnDictEntry = match CnEnDictEntry::lookup_phrase(db.clone(), phrase.clone(), cn_type.clone()) {
            Some(res) => res,
            None => {
                cn_type = CnType::Simplified;
                match CnEnDictEntry::lookup_phrase(db.clone(), phrase.clone(), cn_type.clone()) {
                    Some(res) => res,
                    None => CnEnDictEntry::generate_empty_phrase()
                }
            }
        };
        let phrase_string = match cn_type {
            CnType::Traditional => String::from(&phrase.trad),
            CnType::Simplified => String::from(&phrase.simp),
        };
        let new_vocab = UserVocab { username, from_doc_title, phrase, phrase_string, cn_type };
        return new_vocab;
    }

    pub fn try_delete(db: Database, username: String, phrase: String) -> bool {
        let coll = db.collection(USER_VOCAB_COLL_NAME);
        let query_doc = doc! { "username": username, "phrase_string": phrase }; 
        let res = match coll.delete_one(query_doc, None) {
            Ok(delete_res) => delete_res.deleted_count == 1,
            Err(_) => false,
        };
        return res;
    }
}

impl DatabaseItem for UserVocab {
    fn collection_name(&self) -> &str { return USER_VOCAB_COLL_NAME; }
    fn primary_key(&self) -> String {
        // TODO: this is a dual key of username and phrase_string... how to handle?
        return self.username.clone();
    }
}

impl HtmlString {
    pub fn to_string(&self) -> String { return self.0.clone(); }
}

/* Public Functions */
pub fn connect_to_mongodb() -> Result<Database, Error> {
    let options = ClientOptions::builder()
    .hosts(vec![
        StreamAddress {
            hostname: DB_HOSTNAME.into(),
            port: Some(DB_PORT),
        }
    ])
    .build();
    let client = Client::with_options(options)?;
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
    res = res.replace(&['<', '>', '(', ')', ',', '\"', '.', ';', ':', '\''][..], "");
    return res;
}

pub fn get_sandbox_document(db: Database, doc_id: String) -> Option<String> {
    let coll = db.collection(SANDBOX_COLL_NAME);
    let query_doc = doc! { "doc_id": doc_id };
    let res = match coll.find_one(query_doc, None).unwrap() {
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
    cookie.set_path("/");
    return cookie;
}

pub fn get_username_from_cookie(db: Database, cookie_lookup: Option<&Cookie<'static>>) -> Option<String> {
    let mut res = None;
    if let Some(ref login_cookie) = cookie_lookup {
        let jwt_to_check = login_cookie.value();
        res = validate_jwt_and_get_username(db.clone(), jwt_to_check.to_string());
    }
    return res;
}

pub fn check_password(db: Database, username: String, pw_to_check: String) -> bool {
    let coll = db.collection(USER_COLL_NAME);
    let hashed_pw = str_to_hashed_string(pw_to_check.as_str());
    let query_doc = doc! { "username": username, "pw_hash": hashed_pw.clone() };
    let res = match coll.find_one(query_doc, None).unwrap() {
        Some(document) => {
            let saved_hash = document.get("pw_hash").and_then(Bson::as_str).expect("No password was stored");
            saved_hash == hashed_pw
        },
        None => false
    };
    return res;
}

pub fn convert_string_to_tokenized_html(db: Database, s: String) -> HtmlString {
    let tokenized_string = tokenize_string(s).expect("Tokenizer connection error");
    let coll = db.collection(CEDICT_COLL_NAME);

    let mut res = String::new();
    for phrase in tokenized_string.split(",") {
        // For each phrase, lookup as CnEnDictPhrase (2 queries: 1 as Traditional, 1 as Simplified)
        let trad_query = coll.find_one(doc! { "trad": &phrase }, None).unwrap();
        let simp_query = coll.find_one(doc! { "simp": &phrase }, None).unwrap();
        if trad_query == None && simp_query == None {
            let phrase_html = generate_phrase_without_pinyin_html(phrase).to_string();
            res += phrase_html.as_str();
        } else {
            let mut cn_type = CnType::Traditional;
            let cedict_doc = match trad_query {
                Some(doc) => doc,
                None => {
                    cn_type = CnType::Simplified;
                    simp_query.unwrap()
                },
            };
            let entry: CnEnDictEntry = from_bson(Bson::Document(cedict_doc)).unwrap();
            res += entry.as_html(cn_type).to_string().as_str(); 
        }
    }
    return HtmlString(res);
}


pub fn render_document_table(db: Database, username: &str) -> HtmlString {
    // get all documents for user
    let coll = db.collection(USER_DOC_COLL_NAME);
    let mut res = String::new();
    res += "<table>\n";
    res += "<tr><th>Title</th><th>Preview</th></tr>\n";
    let query_doc = doc! { "username": username };
    match coll.find(query_doc, None) {
        Ok(cursor) => {
            // add each document as a <tr> item
            for item in cursor {
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
                let mut content_preview = String::with_capacity((n+1)*3 as usize); // FYI: each Chinese char is 3 usize
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



                res += format!("<tr><td>{}</td><td>{}</td><td>{}</td></tr>\n", title, content_preview, delete_button).as_str();
            }
        },
        Err(e) => {
            println!("Error when searching for documents for user {}: {:?}", username, e);
        }
    }
    res += "</table>";
    return HtmlString(res);
}

pub fn render_vocab_table(db: Database, username: &str) -> HtmlString {
    let coll = db.collection(USER_VOCAB_COLL_NAME);
    let mut res = String::new();
    res += "<table>\n";
    res += format!("<tr><th>{}</th><th>{}</th></tr>\n", "Term", "Saved From").as_str();
    let query_doc = doc! { "username": username };
    match coll.find(query_doc, None) {
        Ok(cursor) => {
            // add each document as a <tr> item
            for item in cursor {
                // unwrap BSON document
                let user_doc = item.unwrap(); // TODO handling error case would be better (Result)
                let UserVocab { from_doc_title, phrase, cn_type, .. } = from_bson(Bson::Document(user_doc)).unwrap();
                let CnEnDictEntry { trad, simp, .. } = phrase;
                // TODO add user settings for traditional/simplified config. For now, default to traditional
                // TODO convert doc_title to <a href=...></a> link
                let hanzi = match cn_type {
                    CnType::Traditional => trad,
                    CnType::Simplified => simp
                };
                let delete_button = format!("<a href=\"/api/delete-vocab/{}\">X</a>", &hanzi);
                let row = format!("<tr><td>{}</td><td>{}</td><td>{}</td></tr>\n", &hanzi, &from_doc_title, &delete_button);
                res += &row;
            }
        },
        Err(e) => {
            println!("Error when searching for vocab for user {}: {:?}", username, e);
        }
    }
    res += "</table>";
    return HtmlString(res);
}

// TODO: change this to generic function (check if field exists)
pub fn check_if_username_exists(db: Database, username: &str) -> bool {
    let coll = db.collection(USER_COLL_NAME);
    let username_search = coll.find_one(doc! { "username": username }, None).unwrap();
    return username_search != None;
}

/* Private Functions */
fn insert_one_doc(coll: Collection, doc: Document) -> Result<(), Error> {
    coll.delete_one(doc.clone(), None)?; // remove once can set unique indices
    match coll.insert_one(doc.clone(), None) {
        Ok(_) => {}
        Err(e) => {
            println!("Skipping insert for doc: {:?}\n\tGot error: {:?}", doc, e);
        }
    }
    return Ok(());
}

fn tokenize_string(s: String) -> std::io::Result<String> {
    let mut stream = TcpStream::connect(format!("{}:{}", DB_HOSTNAME, TOKENIZER_PORT))?;
    stream.write(s.as_bytes())?;
    let n_bytes = s.as_bytes().len();
    let mut tokenized_bytes = vec![0; n_bytes * 2]; // max size includes original 'n_bytes' + at most 'n_bytes' commas
    stream.read(&mut tokenized_bytes)?;

    let mut res = String::from_utf8(tokenized_bytes).unwrap();
    res = res.trim_matches(char::from(0)).to_string(); // remove training '0' chars
    return Ok(res);
}

fn generate_phrase_without_pinyin_html(phrase: &str) -> HtmlString {
    let mut res = String::new();
    res += "<table style=\"display: inline-table;\">";
    res += "<tr></tr>"; // No pinyin found
    let mut phrase_td = String::new();
    for c in phrase.chars() {
        phrase_td += format!("<td>{}</td>", c).as_str();
    }
    res += format!("<tr>{}</tr>", phrase_td).as_str();
    res += "</table>";
    return HtmlString(res);
}

// TODO: change this to generic function (check if field exists)
fn check_if_email_exists(db: Database, email: &str) -> bool {
    let coll = db.collection(USER_COLL_NAME);
    let email_search = coll.find_one(doc! { "email": email }, None).unwrap();
    return email_search != None;
}

fn check_coll_for_existing_key_value(coll: Collection, key: &str, value: &str) -> bool {
    let query = coll.find_one(doc! { key: value }, None).unwrap();
    return query != None;
}