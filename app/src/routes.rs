/*
/// General route handling. 
/// Expected form inputs are stored as Structs and defined above the corresponding route.
/// 
/// routes.rs
/// ├── GET
/// |   └── /
/// |   └── /login
/// |   └── /sandbox
/// |   └── /sandbox/<doc_id>
/// |   └── /feedback
/// |   └── /u/<username>
/// |   └── /u/<username>/<doc_title>
*/

use crate::{
    convert_rawstr_to_string, 
    DatabaseItem,
    auth::{get_username_from_cookie, add_user_cookie_to_context},
    config::JWT_NAME,
    html_rendering,
    models::{
        public::SandboxDoc,
        user::{User, UserDoc, UserVocabList},
    }
};
use mongodb::{
    bson::doc,
    sync::Database
};
use rocket::{
    http::{RawStr, Cookies},
    State,
};
use rocket_contrib::templates::Template;
use std::collections::HashMap;

/* GET */
/// /
#[get("/")]
pub fn index(cookies: Cookies, db: State<Database>) -> Template {
    let mut context: HashMap<&str, String> = HashMap::new();
    add_user_cookie_to_context(&cookies, &db, &mut context);
    return Template::render("index", context);
}

/// /login
#[get("/login")]
pub fn login(cookies: Cookies, db: State<Database>) -> Template {
    let mut context: HashMap<&str, String> = HashMap::new();
    add_user_cookie_to_context(&cookies, &db, &mut context);
    return Template::render("login", context);
}

/// /sandbox
#[get("/sandbox")]
pub fn sandbox(cookies: Cookies, db: State<Database>) -> Template {
    let mut context: HashMap<&str, String> = HashMap::new();
    add_user_cookie_to_context(&cookies, &db, &mut context);
    return Template::render("sandbox", context);
}

/// /sandbox/<doc_id>
#[get("/sandbox/<doc_id>")]
pub fn sandbox_view_doc(db: State<Database>, doc_id: &RawStr) -> Template {
    let mut context: HashMap<&str, &str> = HashMap::new();
    let doc_id = convert_rawstr_to_string(doc_id);
    let query_doc = SandboxDoc::try_lookup_one(&db, 
        doc!{ "doc_id": doc_id }
    ).unwrap();
    let body_html = query_doc.get_str("body_html").unwrap();
    let cn_phonetics = query_doc.get_str("cn_phonetics").unwrap();
    context.insert("cn_phonetics", cn_phonetics);
    if body_html != "" {
        context.insert("paragraph_html", body_html);
    }
    return Template::render("reader", context);
}

/// /feedback
#[get("/feedback")]
pub fn feedback(cookies: Cookies, db: State<Database>) -> Template {
    let mut context: HashMap<&str, String> = HashMap::new();
    add_user_cookie_to_context(&cookies, &db, &mut context);
    return Template::render("feedback", context);
}

/// /u/<raw_username>
#[get("/u/<raw_username>")]
pub fn user_profile(cookies: Cookies, db: State<Database>, raw_username: &RawStr) -> Template {
    let mut context: HashMap<&str, String> = HashMap::new(); // Note: <&str, String> makes more sense than <&str, &str> due to variable lifetimes
    let username = convert_rawstr_to_string(raw_username);
    // Compare username with logged-in username from JWT
    match get_username_from_cookie(&db, cookies.get(JWT_NAME)) {
        Some(s) => { 
            if &s == &username {
                let (cn_type, cn_phonetics) = User::get_user_settings(&db, &username);
                let doc_html = html_rendering::render_document_table(&db, &username);
                let vocab_html = html_rendering::render_vocab_table(&db, &username);
            
                context.insert("doc_table", doc_html);
                context.insert("vocab_table", vocab_html);
                context.insert("cn_type", cn_type.to_string());
                context.insert("cn_phonetics", cn_phonetics.to_string());           
            }
            context.insert("logged_in_username", s);
        },
        None => { }
    }
    if User::check_if_username_exists(&db, &username) == true {
        context.insert("username", username); 
    }
    return Template::render("profile", context);
}

/// /u/<raw_username>/<doc_title>
#[get("/u/<raw_username>/<doc_title>")]
pub fn user_view_doc(cookies: Cookies, db: State<Database>, raw_username: &RawStr, doc_title: &RawStr) -> Template {
    let mut context: HashMap<&str, String> = HashMap::new(); // `String` needed b/c lifetimes
    let username = convert_rawstr_to_string(raw_username);
    // Compare username with logged-in username from JWT
    match get_username_from_cookie(&db, cookies.get(JWT_NAME)) {
        Some(s) => { 
            if &s == &username {
                // Get html to render
                let (cn_type, cn_phonetics) = User::get_user_settings(&db, &username);
                let title = convert_rawstr_to_string(doc_title);
                let doc_html_res = UserDoc::try_lookup_one(&db, 
                    doc!{ "username": &username, "title": &title})
                    .unwrap();
                let doc_html = doc_html_res.get_str("body_html").unwrap();
                let user_vocab_res = UserVocabList::try_lookup_one(&db, 
                    doc! { "username": &username, "cn_type": cn_type.as_str() })
                    .unwrap();
                let user_char_list_string = user_vocab_res.get_str("unique_char_list").unwrap();
                let user_uid_list_string = user_vocab_res.get_str("unique_uid_list").unwrap();
                context.insert("paragraph_html", String::from(doc_html));
                context.insert("user_char_list_string", String::from(user_char_list_string));
                context.insert("user_uid_list_string", String::from(user_uid_list_string));
                context.insert("cn_phonetics", cn_phonetics.to_string());
            }
        },
        None =>  { context.insert("paragraph_html", String::from("<p>Not authenticated as user</p>")); }
    }
    if User::check_if_username_exists(&db, &username) == true {
        context.insert("username", username); 
    }
    return Template::render("reader", context);
}