/*
/// Trait definitions and General purpose helper functions
/// 
/// lib.rs
/// ├── CacheItem: Trait
/// ├── DatabaseItem: Trait
/// |
/// └── pub fn:
///     └── connect_to_mongodb
///     └── connect_to_redis
///     └── convert_rawstr_to_string
///     └── launch_rocket
*/

#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;

pub mod auth;
pub mod config;
pub mod html;
pub mod models;
pub mod routes;

use crate::config::{DB_NAME, DB_URI, REDIS_URI};
use mongodb::{
    bson::{self, doc, document::Document, Bson},
    sync::Database
};
use rocket::http::RawStr;
use rocket_contrib::{
    templates::Template,
    serve::StaticFiles
};
use serde::Serialize;
use std::error::Error;
use redis::aio::Connection;
use tokio::runtime::Runtime;

/* Traits */
/// An object that can be found in Redis (using a uid).
/// TODO: add required implementation for uid_keys (return uid keys as vec, at worst for documentation)
pub trait CacheItem {
    /* Default-Enabled */
    /// This removes all spaces, then concatenates the elements in ordered_items.
    /// This uid is used for lookup in the cache.
    /// A uid schema must be manually verified to produce no cache collisions.
    fn generate_uid(ordered_items: Vec<&str>) -> String {
        let mut res = String::with_capacity(100); // 80 chars is upper-bound from largest CEDICT entry
        for item in ordered_items {
            res += &item.replace(" ", "");
        }
        return res;
    }
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
    /// Attempts to insert the object into MongoDB.
    fn try_insert(&self, db: &Database) -> Result<String, Box<dyn Error>> where Self: Serialize {
        let coll = (*db).collection(Self::collection_name());
        let new_doc = self.as_document();
        match coll.insert_one(new_doc, None) {
            Ok(_) => {}
            Err(e) => { return Err(Box::new(e)); }
        }
        return Ok(self.primary_key().to_string());
    }
    /// If the current object exists in MongoDB, attempts to update the corresponding key field with a new value.
    fn try_update(&self, db: &Database, key: &str, new_value: &str) -> Result<String, Box<dyn Error>> where Self: Serialize {
        let coll = (*db).collection(Self::collection_name());
        let update_doc = doc! { key: new_value };
        let update_query = doc! { "$set": update_doc };
        match coll.update_one(self.as_document(), update_query, None) {
            Ok(_) => {},
            Err(e) => { return Err(Box::new(e)); }
        }
        return Ok(self.primary_key().to_string());
    }

    /* Requires Implementation */
    /// Returns the collection name in MongoDB where the objects should be stored.
    fn collection_name() -> &'static str;
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