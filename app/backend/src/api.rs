/*
/// Route handling for authenticated users.
/// Expected form inputs are stored as Structs and defined above the corresponding route.
/// Note: the `/api/` base dir is appended when route is mounted in lib.rs.
///
/// api.rs
/// ├── GET
/// |   └── /api/sandbox/{doc_id}
/// |   └── /api/delete-doc
/// |   └── /api/delete-vocab
/// |   └── /api/logout
/// |   └── /api/docs-to-csv
/// |   └── /api/vocab-to-csv
/// |
/// └── POST
///     └── /api/feedback
///     └── /api/auth
///     └── /api/upload
///     └── /api/vocab
///     └── /api/update_settings
*/

use crate::{
    convert_rawstr_to_string, 
    DatabaseItem,
    auth::{generate_http_cookie, get_username_from_cookie},
    config::JWT_NAME,
    models::{
        sandbox::{AppFeedback, SandboxDoc},
        user::{User, UserDoc, UserVocab},
        zh::{CnType, CnPhonetics}
    }
};
use mongodb::{
    bson::doc,
    sync::Database
};
use rocket::{
    http::{RawStr, Cookie, Cookies, Status},
    request::{Form, LenientForm},
    State,
};
use rocket_contrib::{json, json::{Json, JsonValue}};
use tokio::runtime::Handle;

// /* GET */
// TODO: Add the GET API for getting a user doc (previous deferred to html.rs)
// TODO: Add tokenization route service

/// /api/sandbox/<doc_id>
#[get("/sandbox/<doc_id>")]
pub fn sandbox(db: State<Database>, doc_id: &RawStr) -> Json<JsonValue> {
    let doc_id = convert_rawstr_to_string(doc_id);
    let query_vec = SandboxDoc::get_values_from_query(&db, 
        doc!{ "doc_id": doc_id },
        vec!["body_html", "cn_phonetics"]
    );
    return Json(json!({
        "body_html": query_vec[0].to_owned(),
        "cn_phonetics": query_vec[1].to_owned(),
    }));
}
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
/// /api/logout
#[get("/logout")]
pub fn logout(mut cookies: Cookies) -> Status {
    let mut removal_cookie = Cookie::named(JWT_NAME);
    removal_cookie.set_path("/");
    cookies.remove(removal_cookie);
    return Status::Ok;
}

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
    let fields: Vec<&str> = vec!["title", "body", "source", "created_on"];
    let field_vals = UserDoc::aggregate_all_values_from_query(&db, query_doc, fields);
    return Json(json!({
        "title": field_vals[0].to_owned(),
        "body": field_vals[1].to_owned(),
        "source": field_vals[2].to_owned(),
        "created_on": field_vals[3].to_owned()
    }));
}

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
/// Matches definition in feedback.html.tera.
#[derive(FromForm)]
pub struct AppFeedbackForm<'f> {
    feedback: &'f RawStr,
    contact: &'f RawStr,
}
/// /api/feedback
#[post("/feedback", data = "<user_feedback>")]
pub fn feedback(db: State<Database>, user_feedback: Form<AppFeedbackForm<'_>>) -> Status {
    let AppFeedbackForm { feedback, contact } = user_feedback.into_inner();
    let feedback = convert_rawstr_to_string(feedback);
    let contact = convert_rawstr_to_string(contact);
    let new_feedback = AppFeedback::new(feedback, contact);
    let res = match new_feedback.try_insert(&db) {
        Ok(_) => Status::Accepted,
        Err(_) => Status::InternalServerError
    };
    return res;
}

/// Matches definition in ../../frontend/src/login.rs
#[derive(FromForm)]
pub struct UserAuthForm<'f> {
    username: &'f RawStr,
    password: &'f RawStr,
    email: &'f RawStr,
}
/// /api/auth
#[post("/auth", data = "<user_input>")]
pub fn auth(mut cookies: Cookies, db: State<Database>, user_input: LenientForm<UserAuthForm<'_>>) -> Status {
    let UserAuthForm { username, password, email } = user_input.into_inner();
    let username = convert_rawstr_to_string(username);
    let password = convert_rawstr_to_string(password);
    let email = convert_rawstr_to_string(email);
    let res_status = match email == "" {
        true => {
            let new_user = User::new(username.clone(), password.clone(), email);
            match new_user.try_insert(&db) {
                Ok(_) => {
                    let new_cookie = generate_http_cookie(&db, username, password);
                    cookies.add(new_cookie);
                    Status::Accepted
                },
                Err(_) => Status::InternalServerError
            }
        },
        false => {
            let is_valid_password = User::check_password(&db, &username, &password);
            match is_valid_password {
                true => {
                    let new_cookie = generate_http_cookie(&db, username, password);
                    cookies.add(new_cookie);
                    Status::Accepted
                },
                false => Status::Unauthorized
            }
        }
    };
    return res_status;
}
/// LenientForm that is used in Sandbox and Profile pages
#[derive(FromForm)]
pub struct UploadForm<'f> {
    title: &'f RawStr,
    source: &'f RawStr,
    body: &'f RawStr,
    url: &'f RawStr,
    cn_type: &'f RawStr,
    cn_phonetics: &'f RawStr,
}
/// /api/upload
#[post("/upload", data="<upload_doc>")]
pub fn upload(cookies: Cookies, db: State<Database>, rt: State<Handle>, upload_doc: LenientForm<UploadForm<'_>>) -> Status {
    let UploadForm { title, source, body, url, cn_type, cn_phonetics } = upload_doc.into_inner();
    let title = convert_rawstr_to_string(title);
    let body = convert_rawstr_to_string(body);
    let source = convert_rawstr_to_string(source);
    let url = convert_rawstr_to_string(url);
    let cn_type = convert_rawstr_to_string(cn_type);
    let cn_phonetics = convert_rawstr_to_string(cn_phonetics);
    // Parse url if available
    let use_url = match url.as_str() {
        "" => false,
        _ => &body == ""
    };
    let username_from_cookie = get_username_from_cookie(&db, cookies.get(JWT_NAME));
    let res_status = match username_from_cookie {
        Some(username) => { 
            let new_doc = match use_url {
                false => rt.block_on(UserDoc::new(&db, username, title, body, source)),
                true => rt.block_on(UserDoc::from_url(&db, username, url))
            };
            match new_doc.try_insert(&db) {
                Ok(_) => Status::Accepted,
                Err(e) => {
                    eprintln!("Exception when inserting doc: {:?}", e);
                    Status::UnprocessableEntity
                }
            }
        },
        None => {
            let new_doc = match use_url {
                false => rt.block_on(SandboxDoc::new(body, cn_type, cn_phonetics, source)),
                true => rt.block_on(SandboxDoc::from_url(url, cn_type, cn_phonetics))
            };
            match new_doc.try_insert(&db) {
                Ok(_) => Status::Accepted,
                Err(e) => {
                    eprintln!("Exception when inserting doc: {:?}", e);
                    Status::UnprocessableEntity
                }
            }
        }
    };
    return res_status;
}

/// Matches definition in template.js (primarily called in reader.html.tera).
#[derive(FromForm)]
pub struct UserVocabForm<'f> {
    phrase_uid: &'f RawStr,
    from_doc_title: &'f RawStr,
}
/// /api/vocab
#[post("/vocab", data="<user_vocab>")]
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
            eprintln!("Error: no username found from cookie");
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