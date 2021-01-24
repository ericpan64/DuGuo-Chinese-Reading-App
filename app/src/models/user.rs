/*
/// Data Structures related to a user account
/// 
/// users.rs
/// ├── User: Strict
/// ├── UserDoc: Struct
/// ├── UserVocab: Struct
/// └── UserVocabList: Struct
*/

use chrono::Utc;
use crate::{
    DatabaseItem,
    auth::str_to_hashed_string,
    config::{USER_COLL_NAME, USER_DOC_COLL_NAME, USER_VOCAB_COLL_NAME, USER_VOCAB_LIST_COLL_NAME},
    connect_to_redis,
    html as html_rendering,
    models::zh::{CnType, CnPhonetics, CnEnDictEntry}
};
use mongodb::{
    bson::{doc, Bson, from_bson},
    sync::Database
};
use reqwest;
use scraper;
use serde::{Serialize, Deserialize};
use std::error::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    username: String,
    pw_hash: String,
    email: String,
    cn_type: CnType,
    cn_phonetics: CnPhonetics,
    created_on: String
}

impl DatabaseItem for User {
    fn try_insert(&self, db: &Database) -> Result<String, Box<dyn Error>> {
        let coll = (*db).collection(USER_COLL_NAME);
        let can_register = User::check_if_username_and_email_are_available(db, &self.username, &self.email);
        if can_register {
            let new_doc = self.as_document();
            match coll.insert_one(new_doc, None) {
                Ok(_) => { },
                Err(e) => { return Err(Box::new(e)); }
            }
        }
        return Ok(self.primary_key().to_string());
    }
    fn collection_name(&self) -> &str { return USER_COLL_NAME; }
    fn primary_key(&self) -> &str { return &self.username; }
}

impl User {
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

    pub fn update_user_settings(db: &Database, username: &str, cn_type: Option<CnType>, cn_phonetics: Option<CnPhonetics>) -> Result<(), Box<dyn Error>> {
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

    fn default_settings() -> (CnType, CnPhonetics) {
        return (CnType::Simplified, CnPhonetics::Pinyin);
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserDoc {
    username: String,
    pub title: String,
    body: String,
    body_html: String,
    // If none, String::new()
    pub source: String, 
    cn_type: CnType,
    cn_phonetics: CnPhonetics,
    pub created_on: String
}

impl DatabaseItem for UserDoc {
    fn collection_name(&self) -> &str { return USER_DOC_COLL_NAME; }
    /// Note: this is not unique per document, a unique primary_key is username + title.
    fn primary_key(&self) -> &str { return &self.username; }
}

impl UserDoc {
    pub async fn new(db: &Database, username: String, desired_title: String, body: String, source: String) -> Self {
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
            false => desired_title
        };
        let created_on = Utc::now().to_string();
        let new_doc = UserDoc { username, title, body, body_html, source, cn_type, cn_phonetics, created_on };
        return new_doc;
    }

    pub async fn from_url(db: &Database, username: String, url: String) -> Self {
        // make request
        let resp = reqwest::get(&url).await.unwrap()
            .text().await.unwrap();
        let html = scraper::Html::parse_document(&resp);
        // get title
        let title_selector = scraper::Selector::parse("title").unwrap();
        let title_text: String = html.select(&title_selector)
            .next().unwrap()
            .text().collect();
        // get body from all headers, paragraphs in-order
        let body_selector = scraper::Selector::parse("body h1,h2,h3,h4,h5,h6,p").unwrap();
        let mut body_text = String::with_capacity(resp.len());
        for item in  html.select(&body_selector) {
            body_text += &item.text().collect::<String>();
        }
        return UserDoc::new(db, username, title_text, body_text, url).await;
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

#[derive(Serialize, Deserialize, Debug)]
pub struct UserVocab {
    username: String,
    pub from_doc_title: String,
    cn_type: CnType,
    cn_phonetics: CnPhonetics,
    pub phrase: String,
    def: String, 
    /// If pinyin, formatted_pinyin
    phrase_phonetics: String, 
    pub phrase_html: String,
    pub created_on: String
}

impl DatabaseItem for UserVocab {
    fn try_insert(&self, db: &Database) -> Result<String, Box<dyn Error>> where Self: Serialize {
        let coll = (*db).collection(self.collection_name());
        let new_doc = self.as_document();
        match coll.insert_one(new_doc, None) {
            Ok(_) => { UserVocabList::append_to_user_vocab_list(db, &self.username, &self.phrase, self.cn_type.as_str())?; },
            Err(e) => { return Err(Box::new(e)); }
        }
        return Ok(self.primary_key().to_string());
    }

    fn collection_name(&self) -> &str { return USER_VOCAB_COLL_NAME; }
    fn primary_key(&self) -> &str { return &self.phrase_html; }
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

#[derive(Serialize, Deserialize, Debug)]
pub struct UserVocabList {
    username: String,
    /// Comma-delimited String
    unique_phrase_list: String, 
    cn_type: CnType
}

impl DatabaseItem for UserVocabList {
    fn collection_name(&self) -> &str { return USER_VOCAB_LIST_COLL_NAME; }
    /// NOTE: this is not necessarily unique per user, a unique primary key is username + cn_type
    fn primary_key(&self) -> &str { return &self.username; } 
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

    fn append_to_user_vocab_list(db: &Database, username: &str, new_phrase: &str, cn_type_str: &str) -> Result<(), Box<dyn Error>> {
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
    
    fn remove_from_user_vocab_list(db: &Database, username: &str, phrase_to_remove: &str, cn_type: &CnType) -> Result<(), Box<dyn Error>> {
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