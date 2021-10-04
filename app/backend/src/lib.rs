/*
/// Trait definitions and General purpose helper functions.
*/

#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;

pub mod auth;
pub mod config;
pub mod models;
pub mod api;
pub mod routes;

use crate::{
    config::{DB_URI, DB_NAME, REDIS_URI, TOKENIZER_HOSTNAME, TOKENIZER_PORT},
    models::{
        zh::{CnEnDictEntry, CnPhrase}
    }
};
use mongodb::{
    bson::{self, doc, document::Document, Bson},
    sync::Database
};
use reqwest;
use rocket::http::RawStr;
use rocket_contrib::{
    serve::StaticFiles
};
use serde::Serialize;
use scraper;
use std::{
    error::Error,
    marker::Sized,
    io::prelude::*,
    net::TcpStream
};
use redis::aio::Connection;
use tokio::runtime::Runtime;

/* Traits */
/// An object that can be found in Redis (using a uid).
pub trait CacheItem {
    /* Default-Enabled */
    /// Gets the uid used in cache lookups.
    fn get_uid(&self) -> String {
        let ordered_items = self.get_uid_ordered_values();
        return Self::generate_uid(ordered_items);
    }
    /// Generates the uid used in cache lookups.
    /// This removes all spaces, then concatenates the elements in ordered_items.
    fn generate_uid(ordered_items: Vec<&str>) -> String {
        let mut res = String::with_capacity(100); // 80 chars is upper-bound from largest CEDICT entry
        for item in ordered_items {
            res += &item.replace(" ", "");
        }
        return res;
    }
    /* Requires Implementation */
    /// From object, returns values to generate uid.
    fn get_uid_ordered_values(&self) -> Vec<&str>;
}

/// An object that can be found in MongoDB.
/// All DatabaseItem object fields are stored into MongoDB as String fields.
pub trait DatabaseItem {
    /* Default-Enabled */
    /// Returns object as bson::document::Document type.
    fn as_document(&self) -> Document where Self: Serialize {
        return bson::to_document(self).unwrap();
    }
    /// Returns object as bson::Bson type.
    fn as_bson(&self) -> Bson where Self: Serialize {
        return bson::to_bson(self).unwrap();
    }
    /// Attempts to lookup a full Document based on the provided query Document.
    fn try_lookup_one(db: &Database, query_doc: Document) -> Option<Document> where Self: Sized {
        let coll = (*db).collection(Self::collection_name());
        return coll.find_one(query_doc, None).unwrap();
    }
    /// Attempts to lookup Vec<Document> based on the provided query Document.
    fn try_lookup_all(db: &Database, query_doc: Document) -> Option<Vec<Document>> where Self: Sized {
        let coll = (*db).collection(Self::collection_name());
        let cursor = coll.find(query_doc, None).unwrap();
        let mut docs: Vec<Document> = Vec::new();
        for doc_res in cursor {
            docs.push(doc_res.unwrap());
        }
        let res = match docs.len() {
            0 => None,
            _ => Some(docs)
        };
        return res;
    }
    /// Attempts to insert the object into MongoDB.
    fn try_insert(&self, db: &Database) -> Result<String, Box<dyn Error>> where Self: Serialize {
        let coll = (*db).collection(Self::collection_name());
        let new_doc = self.as_document();
        coll.insert_one(new_doc, None)?;
        return Ok(String::from(self.primary_key()));
    }
    /// If the current object exists in MongoDB, attempts to update the corresponding key fields with new values.
    /// Input Vec<&str> are read in-order and must be the same size.
    fn try_update(&self, db: &Database, keys: Vec<&str>, new_values: Vec<&str>) -> Result<(), Box<dyn Error>> where Self: Serialize {
        let coll = (*db).collection(Self::collection_name());
        let mut update_doc = Document::new();
        let valid_keys = Self::all_field_names();
        if keys.len() == new_values.len() {
            for i in 0..keys.len() {
                if valid_keys.contains(&keys[i]) {
                    update_doc.insert(keys[i], new_values[i]);
                }
            }
        }
        let update_query = doc! { "$set": update_doc };
        coll.update_one(self.as_document(), update_query, None)?;
        return Ok(());
    }
    /// From the first result matching the query_doc, returns the values from input fields
    /// as a Vec<String> (with matching indices as the input fields).
    /// In the case of a failed lookup, a single item (String::new()) is returned in the Vec.
    /// Thus, the len of resulting Vec is always == the len of the fields Vec.
    fn get_values_from_query(db: &Database, query_doc: Document, fields: Vec<&str>) -> Vec<String> {
        let coll = (*db).collection(Self::collection_name());
        let valid_fields = Self::all_field_names();
        let mut res_vec: Vec<String> = Vec::with_capacity(fields.len());
        if let Some(doc) = coll.find_one(query_doc, None).unwrap() {
            for key in fields {
                if valid_fields.contains(&key) {
                    res_vec.push(String::from(doc.get_str(key).unwrap()));
                } else {
                    res_vec.push(String::new());
                }
            }
        } else {
            for _ in fields {
                res_vec.push(String::new());
            }
        }
        return res_vec;
    }
    /// From all documents matching the query_doc, aggregates the values from the input fields
    /// into a Vec<String> (with matching indices as the input fields).
    /// If an input field is invalid, then the corresponding Vec<String> will be empty.
    fn aggregate_all_values_from_query(db: &Database, query_doc: Document, fields: Vec<&str>) -> Vec<Vec<String>> {
        let mut res_vec: Vec<Vec<String>> = Vec::with_capacity(fields.len());
        for _ in 0..fields.len() {
            res_vec.push(Vec::<String>::new());
        }
        let coll = (*db).collection(Self::collection_name());
        let valid_fields = Self::all_field_names();
        let cursor = coll.find(query_doc, None).unwrap();
        for doc_ok in cursor {
            let doc = doc_ok.unwrap();
            for i in 0..fields.len() {
                let key = fields[i];
                if valid_fields.contains(&key) {
                    res_vec[i].push(String::from(doc.get_str(key).unwrap()));
                }
            }
        }
        return res_vec;
    }
    /* Requires Implementation */
    /// Returns the collection name in MongoDB where the objects should be stored.
    fn collection_name() -> &'static str;
    /// Returns all valid field names for the document.
    fn all_field_names() -> Vec<&'static str>;
    /// Returns a generally-informative key for the document.
    /// This is not guaranteed to be unique (though generally is).
    fn primary_key(&self) -> &str;
}

/* Public Functions */
/// Connects to MongoDB (locally: Docker Container, in production: mongoDB Atlas). Connection is handled in main.rs.
pub fn connect_to_mongodb() -> Result<Database, Box<dyn Error>> {
    let client = mongodb::sync::Client::with_uri_str(DB_URI)?;
    let db: Database = client.database(DB_NAME);
    return Ok(db);
}

/// Uses URI to connect to Redis (Docker Container). Connections are primarily in lib.rs and html.rs.
async fn connect_to_redis() -> Result<Connection, Box<dyn Error>> {
    let client = redis::Client::open(REDIS_URI)?;
    let conn = client.get_async_connection().await?;
    return Ok(conn);
}

/// Sanitizes user input. Chinese punctuation is unaffected by this.
pub fn convert_rawstr_to_string(s: &RawStr) -> String {
    let mut res = s.url_decode_lossy().to_string(); // � for invalid UTF-8
    res = res.replace(&['<', '>', '(', ')', '!', '\"', '\'', '\\', ';', '{', '}', '*'][..], "");
    return res;
}

/// Scrapes relevant text from HTML. Returns: (Title, Body Text).
pub async fn scrape_text_from_url(url: &str) -> (String, String) {
    let resp = reqwest::get(url).await.unwrap()
        .text().await.unwrap();
    let html = scraper::Html::parse_document(&resp);
    let title_selector = scraper::Selector::parse("title").unwrap();
    let title_text: String = html.select(&title_selector)
        .next().unwrap()
        .text().collect();
    // Grabs body headers+paragraphs in-order
    let body_selector = scraper::Selector::parse("body h1,h2,h3,h4,h5,h6,p").unwrap();
    let mut body_text = String::with_capacity(resp.len());
    for item in html.select(&body_selector) {
        body_text += &item.text().collect::<String>();
    }
    return (title_text, body_text);
}

/// Renders the HTML using the given CnType and CnPhonetics.
/// Refer to tokenizer_string() for formatting details.
pub async fn convert_string_to_tokenized_phrases(s: &str) -> Vec<CnPhrase> {
    const PHRASE_DELIM: char = '$';
    const PINYIN_DELIM: char = '`';
    let mut conn = connect_to_redis().await.unwrap();
    let tokenized_string = tokenize_string(s.to_string()).expect("Tokenizer connection error");
    let n_phrases = tokenized_string.matches(PHRASE_DELIM).count();
    // Estimate pre-allocated size: max ~2100 chars per phrase (conservitively 2500), 1 usize per char
    let mut res = Vec::with_capacity(n_phrases);
    for token in tokenized_string.split(PHRASE_DELIM) {
        let token_vec: Vec<&str> = token.split(PINYIN_DELIM).collect();
        let raw_phrase = token_vec[0].to_string(); // If Chinese, then Simplified
        let raw_phonetics = token_vec[1].to_string();
        let uid = CnEnDictEntry::generate_uid(vec![&raw_phrase,&raw_phonetics]);
        let entry = CnEnDictEntry::from_uid(&mut conn, uid).await;
        let lookup_success = entry.lookup_succeeded();
        let curr_phrase = CnPhrase {
            entry,
            lookup_success,
            raw_phrase,
            raw_phonetics
        };
        res.push(curr_phrase);
    }
    return res;
}

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
/// Starts the Rocket web server and corresponding services. Called in main.rs.
/// Note: the Tokio version is deliberately set to 0.2.24 to match the MongoDB 1.1.1 driver.
/// No new Tokio runtimes should be created in other functions and since they can lead to runtime panics.
pub fn launch_rocket() -> Result<(), Box<dyn Error>> {
    let db = connect_to_mongodb()?;
    let runtime = Runtime::new().unwrap();
    let rt = runtime.handle().clone();
    rocket::ignite()
        .manage(db)
        .manage(rt)
        .mount("/api/", routes![
            api::get_sandbox_doc,
            api::get_user_doc,
            api::get_user_lists,
            api::delete_user_doc,
            api::delete_user_vocab,
            api::logout,
            api::docs_to_csv,
            api::vocab_to_csv,
            api::feedback,
            api::login,
            api::register,
            api::upload_sandbox_doc,
            api::upload_user_doc,
            api::upload_vocab,
            api::update_settings,
            ])
        .mount("/", StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../frontend/html")).rank(2))
        .mount("/", routes![
            routes::login,
            routes::feedback,
            routes::sandbox,
        ])
        .mount("/static", StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../static")).rank(1))
        .launch();
    return Ok(());
}