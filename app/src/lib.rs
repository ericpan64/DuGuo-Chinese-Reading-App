/*
/// Trait definitions and General purpose helper functions.
/// 
/// lib.rs
/// ├── CacheItem: Trait
/// ├── DatabaseItem: Trait
/// |
/// └── pub fn:
///     └── connect_to_mongodb
///     └── connect_to_redis
///     └── convert_rawstr_to_string
///     └── scrape_text_from_url
///     └── launch_rocket
*/

#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;

/// Module handling user authentication and cookies.
pub mod auth;
/// Module with config &str values.
pub mod config;
/// Module that handles HTML rendering. Often used with alias "html_rendering".
pub mod html;
/// Module defining all data structures and associated functions.
pub mod models;
/// Module defining all of the Rocket web endpoints.
pub mod routes;

use crate::config::{DB_NAME, DB_URI, REDIS_URI};
use mongodb::{
    bson::{self, doc, document::Document, Bson},
    sync::Database
};
use reqwest;
use rocket::http::RawStr;
use rocket_contrib::{
    templates::Template,
    serve::StaticFiles
};
use serde::Serialize;
use scraper;
use std::{
    error::Error,
    marker::Sized
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
    fn try_lookup(db: &Database, query_doc: Document) -> Option<Document> where Self: Sized {
        let coll = (*db).collection(Self::collection_name());
        return coll.find_one(query_doc, None).unwrap();
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

/// Starts the Rocket web server and corresponding services. Called in main.rs.
/// Note: the Tokio version is deliberately set to 0.2.24 to match the MongoDB 1.1.1 driver.
/// No new Tokio runtimes should be created in other functions and since they can lead to runtime panics.
pub fn launch_rocket() -> Result<(), Box<dyn Error>> {
    let db = connect_to_mongodb()?;
    let rt = Runtime::new().unwrap();
    let rt = rt.handle().clone();
    rocket::ignite()
        .attach(Template::fairing())
        .manage(db)
        .manage(rt)
        .mount("/", routes![
            routes::primary::index, 
            routes::primary::login, 
            routes::primary::sandbox, 
            routes::primary::sandbox_upload, 
            routes::primary::sandbox_url_upload, 
            routes::primary::sandbox_view_doc, 
            routes::primary::feedback, 
            routes::primary::feedback_form,
            routes::users::login_form, 
            routes::users::register_form, 
            routes::users::user_profile, 
            routes::users::user_doc_upload, 
            routes::users::user_url_upload,
            routes::users::user_view_doc,
            routes::users::user_vocab_upload,
            routes::users::delete_user_doc,
            routes::users::delete_user_vocab,
            routes::users::update_settings,
            routes::users::documents_to_csv_json,
            routes::users::vocab_to_csv_json,
            routes::users::logout_user])
        .mount("/static", StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/static")))
        .launch();
    return Ok(());
}