/*
/// General route handling
/// 
/// primary.rs
/// ├── GET
/// |   └── /
/// |   └── /login
/// |   └── /sandbox
/// |   └── /sandbox/<doc_id>
/// |   └── /feedback
/// |
/// └── POST
///     └── /api/sandbox-upload
///     └── /api/sandbox-url-upload
///     └── /api/feedback
*/

use crate::{
    convert_rawstr_to_string, 
    DatabaseItem,
    auth::add_user_cookie_to_context,
    models::sandbox::{SandboxDoc, UserFeedback}
};
use mongodb::sync::Database;
use rocket::{
    http::{RawStr, Cookies},
    request::Form,
    response::Redirect,
    State,
};
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use tokio::runtime::Handle;

/* GET */
#[get("/")]
pub fn index(cookies: Cookies, db: State<Database>) -> Template {
    let mut context: HashMap<&str, String> = HashMap::new();
    add_user_cookie_to_context(&cookies, &db, &mut context);
    return Template::render("index", context);
}

#[get("/login")]
pub fn login(cookies: Cookies, db: State<Database>) -> Template {
    let mut context: HashMap<&str, String> = HashMap::new();
    add_user_cookie_to_context(&cookies, &db, &mut context);
    return Template::render("login", context);
}

#[get("/sandbox")]
pub fn sandbox(cookies: Cookies, db: State<Database>) -> Template {
    let mut context: HashMap<&str, String> = HashMap::new();
    add_user_cookie_to_context(&cookies, &db, &mut context);
    return Template::render("sandbox", context);
}

#[get("/sandbox/<doc_id>")]
pub fn sandbox_view_doc(db: State<Database>, doc_id: &RawStr) -> Template {
    let mut context: HashMap<&str, &str> = HashMap::new();
    let doc_id = convert_rawstr_to_string(doc_id);
    let (html, cn_phonetics) = match SandboxDoc::get_doc_html_and_phonetics_from_id(&db, doc_id) {
        Some((text, phonetics)) => (text, phonetics),
        None => (String::new(), String::new())
    };
    context.insert("cn_phonetics", &cn_phonetics);
    if &html != "" {
        context.insert("paragraph_html", &html);
    }
    return Template::render("reader", context);
}

#[get("/feedback")]
pub fn feedback(cookies: Cookies, db: State<Database>) -> Template {
    let mut context: HashMap<&str, String> = HashMap::new();
    add_user_cookie_to_context(&cookies, &db, &mut context);
    return Template::render("feedback", context);
}

/* POST */
#[derive(FromForm)]
pub struct SandboxForm<'f> {
    text: &'f RawStr,
    cn_type: &'f RawStr,
    cn_phonetics: &'f RawStr,
}

#[post("/api/sandbox-upload", data = "<user_text>")]
pub fn sandbox_upload(db: State<Database>, rt: State<Handle>, user_text: Form<SandboxForm<'_>>) -> Redirect {
    let SandboxForm { text, cn_type, cn_phonetics } = user_text.into_inner();    
    let text_as_string = convert_rawstr_to_string(text);
    let cn_type = convert_rawstr_to_string(cn_type);
    let cn_phonetics = convert_rawstr_to_string(cn_phonetics);
    let new_doc = (rt).block_on(SandboxDoc::new(text_as_string, cn_type, cn_phonetics, None));
    let res_redirect = match new_doc.try_insert(&db) {
        Ok(inserted_id) => Redirect::to(uri!(sandbox_view_doc: inserted_id)),
        Err(_) => Redirect::to(uri!(index))
    };
    return res_redirect;
}

#[derive(FromForm)]
pub struct SandboxUrlForm<'f> {
    url: &'f RawStr,
    cn_type: &'f RawStr,
    cn_phonetics: &'f RawStr,
}

#[post("/api/sandbox-url-upload", data = "<user_url>")]
pub fn sandbox_url_upload(db: State<Database>, rt: State<Handle>, user_url: Form<SandboxUrlForm<'_>>) -> Redirect {
    let SandboxUrlForm { url, cn_type, cn_phonetics } = user_url.into_inner();
    let url = convert_rawstr_to_string(url); // Note: ':' is removed
    let cn_type = convert_rawstr_to_string(cn_type);
    let cn_phonetics = convert_rawstr_to_string(cn_phonetics);
    // read http header if present
    let url = url.replace("http//", "http://");
    let url = url.replace("https//", "https://");
    let new_doc = (rt).block_on(SandboxDoc::from_url(url, cn_type, cn_phonetics));
    let res_redirect = match new_doc.try_insert(&db) {
        Ok(inserted_id) => Redirect::to(uri!(sandbox_view_doc: inserted_id)),
        Err(_) => Redirect::to(uri!(index))
    };
    return res_redirect;
}

#[derive(FromForm)]
pub struct UserFeedbackForm<'f> {
    feedback: &'f RawStr,
    contact: &'f RawStr,
}

#[post("/api/feedback", data = "<user_feedback>")]
pub fn feedback_form(db: State<Database>, user_feedback: Form<UserFeedbackForm<'_>>) -> Redirect {
    let UserFeedbackForm { feedback, contact } = user_feedback.into_inner();
    let feedback = convert_rawstr_to_string(feedback);
    let contact = convert_rawstr_to_string(contact);
    let new_feedback = UserFeedback::new(feedback.clone(), contact.clone());
    match new_feedback.try_insert(&db) {
        Ok(_) => {},
        Err(e) => { println!("Error when submitting feedback {} / contact: {}:\n\t{:?}", &feedback, &contact, e); }
    };
    return Redirect::to(uri!(feedback));
}