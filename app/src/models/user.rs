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
    models::zh::{CnType, CnPhonetics, CnEnDictEntry}
};
use mongodb::{
    bson::{doc, document::Document, Bson, from_bson},
    sync::Database
};
use rand::{self, Rng};
use reqwest;
use scraper;
use serde::{Serialize, Deserialize};
use std::{
    collections::{HashMap, HashSet},
    error::Error
};

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    username: String,
    pw_hash: String,
    pw_salt: String,
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
    /// Generates a new User. Passwords are salted and hashed for security.
    pub fn new(username: String, password: String, email: String) -> Self {
        let pw_salt = User::generate_pw_salt();
        let pw_hash = str_to_hashed_string(&password, &pw_salt);
        let (cn_type, cn_phonetics) = User::default_settings();
        let created_on = Utc::now().to_string();
        let new_user = User { username, pw_hash, pw_salt, email, cn_type, cn_phonetics, created_on };
        return new_user;
    }
    /// Returns true if username exists, false otherwise.
    pub fn check_if_username_exists(db: &Database, username: &str) -> bool {
        let coll = (*db).collection(USER_COLL_NAME);
        return (coll.find_one(doc! {"username": username }, None).unwrap()) != None;
    }
    /// Updates CnType+CnPhonetics settings via username.
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
    /// Gets CnType+CnPhonetics settings from username.
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
    /// Attempts to get a user's random salt.
    /// TODO: make generic
    pub fn get_user_salt(db: &Database, username: &str) -> Result<String, Box<dyn Error>> {
        let coll = (*db).collection(USER_COLL_NAME);
        let found_doc = coll.find_one(doc! { "username": username }, None)?.unwrap();
        return Ok(found_doc.get("pw_salt").and_then(Bson::as_str).unwrap().to_string());
    }
    /// Returns true if password is correct given username, false otherwise.
    pub fn check_password(db: &Database, username: &str, pw_to_check: &str) -> bool {
        let res = match User::from_username(db, username) {
            Some(user) => {
                let hashed_input = str_to_hashed_string(pw_to_check, &user.pw_salt);
                user.pw_hash == hashed_input
            },
            None => false
        };
        return res;
    }
    /// Generates a random salt.
    /// TODO: consider moving this to auth.rs
    fn generate_pw_salt() -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                abcdefghijklmnopqrstuvwxyz\
                                0123456789)(*&^%$#@!~";
        const SALT_LEN: usize = 64;
        let mut rng = rand::thread_rng();
        let pw_salt: String = (0..SALT_LEN)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();
        return pw_salt;
    }
    /// Gets the default settings (biased, since this is what I use!)
    fn default_settings() -> (CnType, CnPhonetics) {
        return (CnType::Simplified, CnPhonetics::Pinyin);
    }
    /// Attempts to lookup a User object via the username.
    fn from_username(db: &Database, username: &str) -> Option<Self> {
        let coll = (*db).collection(USER_COLL_NAME);
        let query_res = coll.find_one(doc! {"username": username}, None).unwrap();
        let res: Option<Self> = match query_res {
            Some(doc) => Some(from_bson(Bson::Document(doc)).unwrap()),
            None => None,
        };
        return res;
    }
    /// Returns true if username and email are available, false otherwise.
    /// TODO: make generic
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
    pub body: String,
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
    /// Generates a new UserDoc. For title collisions, a new title is automatically generated (appended by -#).
    pub fn new(db: &Database, username: String, desired_title: String, body: String, source: String) -> Self {
        let (cn_type, cn_phonetics) = User::get_user_settings(db, &username);
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
        let new_doc = UserDoc { username, title, body, source, cn_type, cn_phonetics, created_on };
        return new_doc;
    }
    /// Generates a new UserDoc with HTML-parsed title + text from the given URL.
    /// TODO: standardize URL calls into a lib.rs function.
    pub async fn from_url(db: &Database, username: String, url: String) -> Self {
        let resp = reqwest::get(&url).await.unwrap()
            .text().await.unwrap();
        let html = scraper::Html::parse_document(&resp);
        let title_selector = scraper::Selector::parse("title").unwrap();
        let title_text: String = html.select(&title_selector)
            .next().unwrap()
            .text().collect();
        let body_selector = scraper::Selector::parse("body h1,h2,h3,h4,h5,h6,p").unwrap();
        let mut body_text = String::with_capacity(resp.len());
        for item in  html.select(&body_selector) {
            body_text += &item.text().collect::<String>();
        }
        return UserDoc::new(db, username, title_text, body_text, url);
    }
    /// Returns the 
    /// TODO: make this generic
    pub fn get_body_from_user_doc(db: &Database, username: &str, title: &str) -> Option<String> {
        let (cn_type, cn_phonetics) = User::get_user_settings(db, username);
        let coll = (*db).collection(USER_DOC_COLL_NAME);
        let query_doc = doc! { "username": username, "title": title, "cn_type": cn_type.as_str(), "cn_phonetics": cn_phonetics.as_str() };
        let doc_body = match coll.find_one(query_doc, None).unwrap() {
            Some(doc) => Some(doc.get("body").and_then(Bson::as_str).unwrap().to_string()),
            None => None
        };
        return doc_body;
    }
    /// Attempts to delete a matching object in MongoDB.
    /// TODO: make this generic -- add a "get_unique_keys_as_ordered_vec" function, have input specify values
    pub fn try_delete(db: &Database, username: &str, title: &str) -> bool {
        let (cn_type, cn_phonetics) = User::get_user_settings(db, username);
        let coll = (*db).collection(USER_DOC_COLL_NAME);
        let query_doc = doc! { "username": username, "title": title, "cn_type": cn_type.as_str(), "cn_phonetics": cn_phonetics.as_str() }; 
        let res = match coll.delete_one(query_doc, None) {
            Ok(_) => {
                match UserVocab::try_delete_all_from_title(db, username, title, &cn_type) {
                    Ok(b) => b,
                    Err(_) => false
                }
            },
            Err(_) => false,
        };
        return res;
    }

    /// For the specified fields in input Vec<&str>, if successful returns map of String->Vec<String>
    /// where each inner Vec<String> represents the list of values for all documents
    /// matching the input query. A Vec upper-bound can be specified.
    /// TODO: this can also be generic!
    pub fn get_doc_fields_as_vectors(db: &Database, query_doc: Document, fields: Vec<&str>, capacity: Option<usize>) -> Result<HashMap<String, Vec<String>>, Box<dyn Error>> {
        let mut res_hmap: HashMap<String, Vec<String>> = HashMap::new();
        for key in &fields {
            match capacity {
                Some(capacity) => { res_hmap.insert(String::from(*key), Vec::<String>::with_capacity(capacity)); },
                None => { res_hmap.insert(String::from(*key), Vec::<String>::new()); }
            }
        }
        let coll = (*db).collection(USER_DOC_COLL_NAME);
        match coll.find(query_doc, None) {
            Ok(cursor) => {
                for item in cursor {
                    let doc = item?;
                    for key in &fields {
                        res_hmap.get_mut(*key)
                            .unwrap()
                            .push(String::from(doc.get_str(key)?));
                    }
                }
            },
            Err(e) => { return Err(Box::new(e)) }
        }
        return Ok(res_hmap);
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserVocab {
    pub uid: String,
    username: String,
    pub from_doc_title: String,
    cn_type: CnType,
    cn_phonetics: CnPhonetics,
    pub phrase: String,
    def: String, 
    phrase_phonetics: String, /// If pinyin: formatted pinyin
    pub phrase_html: String,
    pub created_on: String,
    pub radical_map: String
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
    /// Looks-up UserVocab in Redis cache. If CEDICT match is found, then stores appropriate data.
    /// TODO: see if db call can be consolidated
    pub async fn new(db: &Database, username: String, saved_uid: String, from_doc_title: String) -> Self {
        // For lookup, try user-specified first
        let mut conn = connect_to_redis().await.unwrap();
        let (cn_type, cn_phonetics) = User::get_user_settings(db, &username);
        let uid = saved_uid.clone();
        let entry = CnEnDictEntry::from_uid(&mut conn, saved_uid).await;
        let created_on = Utc::now().to_string();
        let radical_map = (&entry.radical_map).to_string();
        let (phrase, def, phrase_phonetics, phrase_html) = entry.get_vocab_data(&cn_type, &cn_phonetics);
        let new_vocab = UserVocab { 
            uid, username, from_doc_title, def,
            phrase, phrase_phonetics, phrase_html,
            cn_type, cn_phonetics, created_on, radical_map
        };
        return new_vocab;
    }
    /// Attempts delete phrase
    /// TODO: generic!
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
    /// Attempts to delete all UserVocab linked to a given UserDoc.
    pub fn try_delete_all_from_title(db: &Database, username: &str, from_doc_title: &str, cn_type: &CnType) -> Result<bool, Box<dyn Error>> {
        let coll = (*db).collection(USER_VOCAB_COLL_NAME);
        let query_doc = doc! { "username": username, "from_doc_title": from_doc_title };
        let mut res = true;
        match coll.find(query_doc, None) {
            Ok(cursor) => {
                for item in cursor {
                    let doc = item?;
                    let phrase = doc.get_str("phrase")?;
                    if UserVocab::try_delete(db, username, phrase, cn_type) == false {
                        res = false;
                        eprintln!("Error: could not delete phrase: {}", phrase);
                    }
                }
            },
            Err(e) => { return Err(Box::new(e)) }
        }
        return Ok(res);
    }
    /// For all documents matching the query_doc, attempts to a Vec<String> of values for each input field.
    /// For the specified fields in input Vec<&str>, if successful returns map of String->Vec<String>.
    /// Each inner Vec<String> represents the list of values for all documents matching the input query.
    /// A Vec upper-bound can be specified.
    /// TODO: make this generic! And document clearer
    pub fn get_doc_fields_as_vectors(db: &Database, query_doc: Document, fields: Vec<&str>, capacity: Option<usize>) -> Result<HashMap<String, Vec<String>>, Box<dyn Error>> {
        let mut res_hmap: HashMap<String, Vec<String>> = HashMap::new();
        for key in &fields {
            match capacity {
                Some(capacity) => { res_hmap.insert(String::from(*key), Vec::<String>::with_capacity(capacity)); },
                None => { res_hmap.insert(String::from(*key), Vec::<String>::new()); }
            }
        }
        let coll = (*db).collection(USER_VOCAB_COLL_NAME);
        match coll.find(query_doc, None) {
            Ok(cursor) => {
                for item in cursor {
                    let doc = item?;
                    for key in &fields {
                        res_hmap.get_mut(*key)
                            .unwrap()
                            .push(String::from(doc.get_str(key)?));
                    }
                }
            },
            Err(e) => { return Err(Box::new(e)) }
        }
        return Ok(res_hmap);
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserVocabList {
    username: String,
    unique_char_list: String,
    unique_phrase_list: String,
    cn_type: CnType
}

impl DatabaseItem for UserVocabList {
    fn collection_name(&self) -> &str { return USER_VOCAB_LIST_COLL_NAME; }
    /// Note: this is not necessarily unique per user, a unique primary key is username + cn_type
    fn primary_key(&self) -> &str { return &self.username; } 
}

impl UserVocabList {
    /// Gets comma-delimited string unique chars from user-saved UserVocab phrases.
    /// TODO: make this generic
    /// TODO: see if can save a database call for user settings
    pub fn get_user_char_list_string(db: &Database, username: &str) -> Option<String> {
        let (cn_type, _) = User::get_user_settings(db, username);
        let coll = (*db).collection(USER_VOCAB_LIST_COLL_NAME);
        let query_doc = doc! { "username": username, "cn_type": cn_type.as_str() };
        let res = match coll.find_one(query_doc, None) {
            Ok(query_res) => {
                match query_res {
                    Some(doc) => Some(doc.get("unique_char_list").and_then(Bson::as_str).unwrap().to_string()),
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
    /// 
    /// TODO: make first half generic
    pub fn get_phrase_list_as_hashset(db: &Database, username: &str, cn_type: &CnType) -> HashSet<String> {
        let coll = (*db).collection(USER_VOCAB_LIST_COLL_NAME);
        let query_doc = doc! { "username": username, "cn_type": cn_type.as_str() };
        let query_res = match coll.find_one(query_doc, None) {
            Ok(query_res) => {
                match query_res {
                    Some(doc) => Some(doc.get("unique_phrase_list").and_then(Bson::as_str).unwrap().to_string()),
                    None => None
                }            
            },
            Err(_) => None
        };
        let list = match query_res {
            Some(list) => list,
            None => String::new()
        };
        let mut res: HashSet<String> = HashSet::new();
        for c in list.split(',') {
            res.insert(c.to_string());
        }
        return res;
    }
    /// Updates UserVocabList object for given username with information form new_phrase.
    /// TODO: cut down unnecessary nesting (on first coll.find_one())
    fn append_to_user_vocab_list(db: &Database, username: &str, new_phrase: &str, cn_type_str: &str) -> Result<(), Box<dyn Error>> {
        let coll = (*db).collection(USER_VOCAB_LIST_COLL_NAME);
        let query_doc = doc! { "username": username, "cn_type": cn_type_str };
        match coll.find_one(query_doc, None) {
            Ok(query_res) => {
                match query_res {
                    Some(doc) => {
                        // Update existing list
                        let prev_doc: UserVocabList = from_bson(Bson::Document(doc)).unwrap();
                        let mut unique_char_list = prev_doc.unique_char_list.clone();
                        let mut unique_phrase_list = prev_doc.unique_phrase_list.clone();
                        // Add unique chars
                        let phrase_string = String::from(new_phrase);
                        for c in (phrase_string).chars() {
                            if !unique_char_list.contains(c) {
                                unique_char_list += &c.to_string();
                                unique_char_list += ",";
                            }
                        }
                        unique_phrase_list += &phrase_string;
                        unique_phrase_list += ",";
                        // Write to db
                        // TODO: allow try_update to accept multiple fields
                        prev_doc.try_update(db, "unique_char_list", &unique_char_list)?;
                        prev_doc.try_update(db, "unique_phrase_list", &unique_phrase_list)?;
                    }
                    None => {
                        // Create new instance with unique chars
                        let mut unique_char_list = String::with_capacity(50);
                        let mut unique_phrase_list = String::from(new_phrase);
                        for c in (unique_phrase_list).chars() {
                            if !unique_char_list.contains(c) {
                                unique_char_list += &c.to_string();
                                unique_char_list += ",";
                            }
                        }
                        unique_phrase_list += ",";
                        // Write to db
                        let username = username.to_string();
                        let cn_type = CnType::from_str(cn_type_str);
                        let new_doc = UserVocabList { username, unique_char_list, unique_phrase_list, cn_type };
                        new_doc.try_insert(db)?;
                    }
                }
            },
            Err(e) => { eprintln!("Error when searching for pinyin list for user {}: {:?}", username, e); }
        }
        return Ok(());
    }
    /// Removes information in UserVocabList object from username based on phrase_to_remove.
    /// TODO: cut-down on unnecessary nesting (on first coll.find_one())
    fn remove_from_user_vocab_list(db: &Database, username: &str, phrase_to_remove: &str, cn_type: &CnType) -> Result<(), Box<dyn Error>> {
        let coll = (*db).collection(USER_VOCAB_LIST_COLL_NAME);
        let query_doc = doc! { "username": username, "cn_type": cn_type.as_str() };  
        match coll.find_one(query_doc, None) {
            Ok(query_res) => {
                match query_res {
                    Some(doc) => {
                        // Update existing list
                        let prev_doc: UserVocabList = from_bson(Bson::Document(doc)).unwrap();
                        let mut unique_char_list = prev_doc.unique_char_list.clone();
                        // Remove unique chars
                        let phrase_string = String::from(phrase_to_remove);
                        for c in (phrase_string).chars() {
                            if unique_char_list.contains(c) {
                                // remove the string from unique_char_list
                                let c_with_comma = format!("{},", c);
                                unique_char_list = unique_char_list.replace(&c_with_comma, "");
                            }
                        }
                        let phrase_with_comma = format!("{},", phrase_string);
                        let unique_phrase_list = prev_doc.unique_phrase_list.replace(&phrase_with_comma, "");
                        // Write to db
                        // TODO: improve try_update
                        prev_doc.try_update(db, "unique_char_list", &unique_char_list)?;
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