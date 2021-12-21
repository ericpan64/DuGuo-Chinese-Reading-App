/*
/// Launches backend!
*/
#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;

use duguo::*; // lib.rs
use rocket_contrib::{
    templates::Template,
    serve::StaticFiles
};
use tokio::runtime::Runtime;
use std::error::Error;
/// Starts the Rocket web server and corresponding services.
/// Note: the Tokio version is deliberately set to 0.2.24 to match the MongoDB 1.1.1 driver.
/// No new Tokio runtimes should be created in other functions and since they can lead to runtime panics.
pub fn launch_rocket() -> Result<(), Box<dyn Error>> {
    let db = connect_to_mongodb()?;
    let runtime = Runtime::new().unwrap();
    let rt = runtime.handle().clone();
    rocket::ignite()
        .attach(Template::fairing())
        .manage(db)
        .manage(rt)
        .mount("/api/", routes![
            api::get_doc,
            api::get_all_user_items,
            api::get_user_vocab_string,
            api::delete_user_doc,
            api::delete_user_vocab,
            api::logout,
            api::feedback,
            api::login,
            api::register,
            api::upload_sandbox_doc,
            api::upload_user_doc,
            api::upload_vocab,
            api::update_settings,
            ])
        .mount("/", routes![
            routes::index,
            routes::about,
            routes::login,
            routes::sandbox,
            routes::sandbox_doc,
            routes::feedback,
            routes::user_profile,
            routes::user_doc,
        ])
        .mount("/static", StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/static")))
        .launch();
    return Ok(());
}

fn main() {
    launch_rocket().unwrap();
}