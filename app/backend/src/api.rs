/*
/// Route handling for authenticated users.
/// Expected form inputs are stored as Structs and defined above the corresponding route.
/// 
/// users.rs
/// ├── GET
/// |   └── /api/delete-doc/<doc_title>
/// |   └── /api/delete-vocab/<phrase>
/// |   └── /api/logout
/// |   └── /api/docs-to-csv
/// |   └── /api/vocab-to-csv
/// |
/// └── POST
///     └── /api/login
///     └── /api/register
///     └── /api/upload
///     └── /api/url-upload
///     └── /api/vocab
///     └── /api/update-settings
*/

use crate::{
    convert_rawstr_to_string, 
    DatabaseItem,
    auth::{generate_http_cookie, get_username_from_cookie},
    config::JWT_NAME,
    html as html_rendering,
    models::{
        user::{User, UserDoc, UserVocab, UserVocabList},
        zh::{CnType, CnPhonetics}
    }
};
use mongodb::{
    bson::doc,
    sync::Database
};
use rocket::{
    http::{RawStr, Cookie, Cookies, Status},
    request::Form,
    response::Redirect,
    State,
};
use rocket_contrib::{json, json::{Json, JsonValue}};
use serde::Serialize;
use std::collections::HashMap;
use tokio::runtime::Handle;

// /* GET */
// /// /api/sandbox/<doc_id>
// #[get("/api/sandbox/<doc_id>")]
// pub fn sandbox_view_doc(db: State<Database>, doc_id: &RawStr) -> Template {
//     let mut context: HashMap<&str, &str> = HashMap::new();
//     let doc_id = convert_rawstr_to_string(doc_id);
//     let query_vec = SandboxDoc::get_values_from_query(&db, 
//         doc!{ "doc_id": doc_id },
//         vec!["body_html", "cn_phonetics"]
//     );
//     let (body_html, cn_phonetics) = query_vec.iter().next_tuple().unwrap();
//     context.insert("cn_phonetics", &cn_phonetics);
//     if body_html.as_str() != "" {
//         context.insert("paragraph_html", &body_html);
//     }
//     return Template::render("reader", context);
// }
/// /api/delete-doc/<doc_title>
#[get("/delete-doc/<doc_title>")]
pub fn delete_doc(cookies: Cookies, db: State<Database>, rt: State<Handle>, doc_title: &RawStr) -> Status {
    let title = convert_rawstr_to_string(doc_title);
    let username = get_username_from_cookie(&db, cookies.get(JWT_NAME)).unwrap();
    rt.block_on(UserDoc::try_delete(&db, &username, &title));
    return Status::Ok;
}
/// /api/delete-vocab/<vocab_uid>
#[get("/delete-vocab/<vocab_uid>")]
pub fn delete_vocab(cookies: Cookies, db: State<Database>, rt: State<Handle>, vocab_uid: &RawStr) -> Status {
    let phrase_uid = convert_rawstr_to_string(vocab_uid);
    let username = get_username_from_cookie(&db, cookies.get(JWT_NAME)).unwrap();
    let (cn_type, _) = User::get_user_settings(&db, &username);
    rt.block_on(UserVocab::try_delete(&db, &username, &phrase_uid, &cn_type));
    return Status::Ok;
}
// /// /api/logout
// #[get("/api/logout")]
// pub fn logout_user(mut cookies: Cookies) -> Redirect {
//     let mut removal_cookie = Cookie::named(JWT_NAME);
//     removal_cookie.set_path("/");
//     cookies.remove(removal_cookie);
//     return Redirect::to("/");
// }

// #[derive(Serialize)]
// pub struct UserDocCsvList {
//     title: Vec<String>,
//     body: Vec<String>,
//     source: Vec<String>,
//     created_on: Vec<String>
// }
/// /api/docs-to-csv
#[get("/docs-to-csv")]
pub fn docs_to_csv(cookies: Cookies, db: State<Database>) -> Json<JsonValue> {
    let query_doc = match get_username_from_cookie(&db, cookies.get(JWT_NAME)) {
        Some(username) => {
            let (cn_type, cn_phonetics) = User::get_user_settings(&db, &username);
            doc! { "username": username, "cn_type": cn_type.as_str(), "cn_phonetics": cn_phonetics.as_str() }
        },
        None => doc! { "username": "" }
    };
    // upper-bound at 2500 docs (approx match with <=5MB csv limit), update as needed
    let fields: Vec<&str> = vec!["title", "body", "source", "created_on"];
    let field_vals = UserDoc::aggregate_all_values_from_query(&db, query_doc, fields);
    // Matches definition in handleTables.js (called in userprofile.html.tera).
    return Json(json!({
        "title": field_vals[0].to_owned(),
        "body": field_vals[1].to_owned(),
        "source": field_vals[2].to_owned(),
        "created_on": field_vals[3].to_owned()
    }));
}

// #[derive(Serialize)]
// pub struct UserVocabCsvList {
//     phrase: Vec<String>,
//     phrase_phonetics: Vec<String>,
//     def: Vec<String>,
//     from_doc_title: Vec<String>,
//     radical_map: Vec<String>,
//     created_on: Vec<String>
// }
/// /api/vocab-to-csv
#[get("/vocab-to-csv")]
pub fn vocab_to_csv(cookies: Cookies, db: State<Database>) -> Json<JsonValue> {
    let query_doc = match get_username_from_cookie(&db, cookies.get(JWT_NAME)) {
        Some(username) => {
            let (cn_type, cn_phonetics) = User::get_user_settings(&db, &username);
            doc! { "username": username, "cn_type": cn_type.as_str(), "cn_phonetics": cn_phonetics.as_str() }
        },
        None => doc! { "username": "" }
    };
    let fields: Vec<&str> = vec!["phrase", "phrase_phonetics", "def", "from_doc_title", "radical_map", "created_on"];
    let field_vals = UserVocab::aggregate_all_values_from_query(&db, query_doc, fields);
    // Matches definition in handleTables.js (called in userprofile.html.tera).
    return Json(json!({
        "phrase": field_vals[0].to_owned(),
        "phrase_phonetics": field_vals[1].to_owned(),
        "def": field_vals[2].to_owned(),
        "from_doc_title": field_vals[3].to_owned(),
        "radical_map": field_vals[4].to_owned(),
        "created_on": field_vals[5].to_owned()
    }));
}

// /* POST */
// /// Matches definition in sandbox.html.tera.
// #[derive(FromForm)]
// pub struct SandboxForm<'f> {
//     text: &'f RawStr,
//     cn_type: &'f RawStr,
//     cn_phonetics: &'f RawStr,
// }
// /// /api/sandbox-upload
// #[post("/api/sandbox-upload", data = "<user_text>")]
// pub fn sandbox_upload(db: State<Database>, rt: State<Handle>, user_text: Form<SandboxForm<'_>>) -> Redirect {
//     let SandboxForm { text, cn_type, cn_phonetics } = user_text.into_inner();    
//     let text_as_string = convert_rawstr_to_string(text);
//     let cn_type = convert_rawstr_to_string(cn_type);
//     let cn_phonetics = convert_rawstr_to_string(cn_phonetics);
//     let new_doc = rt.block_on(SandboxDoc::new(text_as_string, cn_type, cn_phonetics, String::new()));
//     let res_redirect = match new_doc.try_insert(&db) {
//         Ok(inserted_id) => Redirect::to(uri!(sandbox_view_doc: inserted_id)),
//         Err(_) => Redirect::to(uri!(index))
//     };
//     return res_redirect;
// }
// /// Matches definition in sandbox.html.tera.
// #[derive(FromForm)]
// pub struct SandboxUrlForm<'f> {
//     url: &'f RawStr,
//     cn_type: &'f RawStr,
//     cn_phonetics: &'f RawStr,
// }
// /// /api/sandbox-url-upload
// #[post("/api/sandbox-url-upload", data = "<user_url>")]
// pub fn sandbox_url_upload(db: State<Database>, rt: State<Handle>, user_url: Form<SandboxUrlForm<'_>>) -> Redirect {
//     let SandboxUrlForm { url, cn_type, cn_phonetics } = user_url.into_inner();
//     let url = convert_rawstr_to_string(url);
//     let cn_type = convert_rawstr_to_string(cn_type);
//     let cn_phonetics = convert_rawstr_to_string(cn_phonetics);
//     let new_doc = rt.block_on(SandboxDoc::from_url(url, cn_type, cn_phonetics));
//     let res_redirect = match new_doc.try_insert(&db) {
//         Ok(inserted_id) => Redirect::to(uri!(sandbox_view_doc: inserted_id)),
//         Err(_) => Redirect::to(uri!(index))
//     };
//     return res_redirect;
// }
// /// Matches definition in feedback.html.tera.
// #[derive(FromForm)]
// pub struct AppFeedbackForm<'f> {
//     feedback: &'f RawStr,
//     contact: &'f RawStr,
// }
// /// /api/feedback
// #[post("/api/feedback", data = "<user_feedback>")]
// pub fn feedback_form(db: State<Database>, user_feedback: Form<AppFeedbackForm<'_>>) -> Redirect {
//     let AppFeedbackForm { feedback, contact } = user_feedback.into_inner();
//     let feedback = convert_rawstr_to_string(feedback);
//     let contact = convert_rawstr_to_string(contact);
//     let new_feedback = AppFeedback::new(feedback.clone(), contact.clone());
//     match new_feedback.try_insert(&db) {
//         Ok(_) => {},
//         Err(e) => { println!("Error when submitting feedback {} / contact: {}:\n\t{:?}", &feedback, &contact, e); }
//     };
//     return Redirect::to(uri!(feedback));
// }

/// Matches definition in login.html.tera.
#[derive(FromForm)]
pub struct UserLoginForm<'f> {
    username: &'f RawStr,
    password: &'f RawStr,
}
/// /api/login
#[post("/login", data = "<user_input>")]
pub fn login(mut cookies: Cookies, db: State<Database>, user_input: Form<UserLoginForm<'_>>) -> Status {
    let UserLoginForm { username, password } = user_input.into_inner();
    let username = convert_rawstr_to_string(username);
    let password = convert_rawstr_to_string(password);
    let is_valid_password = User::check_password(&db, &username, &password);
    let res_status = match is_valid_password {
        true => {
            let new_cookie = generate_http_cookie(&db, username, password);
            cookies.add(new_cookie);
            Status::Accepted
        },
        false => Status::Unauthorized
    };
    return res_status;
}
/// Matches definition in login.html.tera.
#[derive(FromForm)]
pub struct UserRegisterForm<'f> {
    username: &'f RawStr,
    email: &'f RawStr,
    password: &'f RawStr,
}
/// /api/register
#[post("/register", data = "<user_input>")]
pub fn register(mut cookies: Cookies, db: State<Database>, user_input: Form<UserRegisterForm<'_>>) -> Status {
    let UserRegisterForm { username, email, password } = user_input.into_inner();
    let username = convert_rawstr_to_string(username);
    let password = convert_rawstr_to_string(password);
    let email = convert_rawstr_to_string(email);
    let new_user = User::new(username.clone(), password.clone(), email);
    let res_status = match new_user.try_insert(&db) {
        Ok(_) => {
            let new_cookie = generate_http_cookie(&db, username, password);
            cookies.add(new_cookie);
            Status::Accepted
        },
        Err(_) => Status::UnprocessableEntity
    };
    return res_status;
}
// /// Matches definition in userprofile.html.tera.
// #[derive(FromForm)]
// pub struct UserDocumentForm<'f> {
//     title: &'f RawStr,
//     source: &'f RawStr,
//     body: &'f RawStr,
// }
// /// /api/upload
// #[post("/api/upload", data="<user_doc>")]
// pub fn user_doc_upload(cookies: Cookies, db: State<Database>, rt: State<Handle>, user_doc: Form<UserDocumentForm<'_>>) -> Redirect {
//     let UserDocumentForm { title, source, body } = user_doc.into_inner();
//     let title = convert_rawstr_to_string(title);
//     let body = convert_rawstr_to_string(body);
//     let source = convert_rawstr_to_string(source);    
//     let username_from_cookie = get_username_from_cookie(&db, cookies.get(JWT_NAME));
//     let res_redirect = match username_from_cookie {
//         Some(username) => { 
//             let new_doc = rt.block_on(UserDoc::new(&db, username, title, body, source));
//             match new_doc.try_insert(&db) {
//                 Ok(username) => Redirect::to(uri!(user_profile: username)),
//                 Err(e) => {
//                     eprintln!("Exception when inserting doc: {:?}", e);
//                     Redirect::to("/")
//                 }
//             }
//         },
//         None => Redirect::to("/")
//     };
//     return res_redirect;
// }
// /// Matches definition in userprofile.html.tera.
// #[derive(FromForm)]
// pub struct UserUrlForm<'f> {
//     url: &'f RawStr,
// }
// /// /api/url-upload
// #[post("/api/url-upload", data = "<user_url>")]
// pub fn user_url_upload(cookies: Cookies, db: State<Database>, rt: State<Handle>, user_url: Form<UserUrlForm<'_>>) -> Redirect {
//     let UserUrlForm { url } = user_url.into_inner();
//     let url = convert_rawstr_to_string(url);
//     let username_from_cookie = get_username_from_cookie(&db, cookies.get(JWT_NAME));
//     let res_redirect = match username_from_cookie {
//         Some(username) => { 
//             let new_doc = rt.block_on(UserDoc::from_url(&db, username, url));
//             match new_doc.try_insert(&db) {
//                 Ok(username) => Redirect::to(uri!(user_profile: username)),
//                 Err(e) => { 
//                     eprintln!("Exception when inserting doc from url: {:?}", e);
//                     Redirect::to("/") 
//                 } 
//             }
//         },
//         None => Redirect::to("/")
//     };
//     return res_redirect;
// }
/// Matches definition in template.js (primarily called in reader.html.tera).
#[derive(FromForm)]
pub struct UserVocabForm<'f> {
    phrase_uid: &'f RawStr,
    from_doc_title: &'f RawStr,
}
/// /api/vocab
#[post("/api/vocab", data="<user_vocab>")]
pub fn vocab(cookies: Cookies, db: State<Database>, rt: State<Handle>, user_vocab: Form<UserVocabForm<'_>>) -> Status {
    let UserVocabForm { phrase_uid, from_doc_title } = user_vocab.into_inner();
    let phrase = convert_rawstr_to_string(phrase_uid);
    let from_doc_title = convert_rawstr_to_string(from_doc_title);
    let username_from_cookie = get_username_from_cookie(&db, cookies.get(JWT_NAME));
    let res_status = match username_from_cookie {
        Some(username) => { 
            let new_vocab = rt.block_on(UserVocab::new(&db, username, phrase, from_doc_title));
            match new_vocab.try_insert(&db) {
                Ok(_) => Status::Accepted,
                Err(_) => Status::ExpectationFailed
            }
        },
        None => {
            println!("Error: no username found from cookie");
            Status::BadRequest
        }
    };
    return res_status;
}
/// Matches definition in userprofile.html.tera.
#[derive(FromForm)]
pub struct UserSettingForm<'f> {
    setting: &'f RawStr,
}
/// /api/update-settings
#[post("/update-settings", data = "<user_setting>")]
pub fn update_settings(cookies: Cookies, db: State<Database>, user_setting: Form<UserSettingForm<'_>>) -> Status {
    let UserSettingForm { setting } = user_setting.into_inner();
    let setting = convert_rawstr_to_string(setting);
    let username_from_cookie = get_username_from_cookie(&db, cookies.get(JWT_NAME));
    let res_status = match username_from_cookie {
        Some(username) => {
            let cn_type = CnType::from_str(&setting);
            let cn_phonetics = CnPhonetics::from_str(&setting);
            match User::update_user_settings(&db, &username, cn_type, cn_phonetics) {
                Ok(_) => Status::Accepted,
                Err(_) => Status::BadRequest
            }
        },
        None => Status::Unauthorized
    };
    return res_status;
}