/*
/// API Route Handling
/// Expected form inputs are stored as Structs and defined above the corresponding route.
/// Note: the `/api/` base dir is appended when route is mounted in lib.rs.
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
    request::Form,
    State,
};
use rocket_contrib::{json, json::{Json, JsonValue}};
use tokio::runtime::Handle;

// /* GET */
/// /api/sandbox/<doc_id>
#[get("/get-sandbox-doc/<doc_id>")]
pub fn get_sandbox_doc(db: State<Database>, doc_id: &RawStr) -> Json<JsonValue> {
    let doc_id = convert_rawstr_to_string(doc_id);
    let res = match SandboxDoc::try_lookup_one(&db, doc!{"doc_id": doc_id}) {
        Some(doc) => doc,
        None => doc!{"error": "No document found"}
    };
    return Json(json!{res});
}
/// /api/get-user-doc/<doc_title>
#[get("/get-user-doc/<doc_title>")]
pub fn get_user_doc(cookies: Cookies, db: State<Database>, doc_title: &RawStr) -> Json<JsonValue> {
    let username = get_username_from_cookie(&db, cookies.get(JWT_NAME)).unwrap();
    let doc_title = convert_rawstr_to_string(doc_title);
    let res = match UserDoc::try_lookup_one(&db, doc!{"username": username, "title": doc_title}) {
        Some(doc) => doc,
        None => doc!{"error": "No document found"}
    };
    return Json(json!{res});
}
/// /api/get-user-lists
#[get("/get-user-lists")]
pub fn get_user_lists(cookies: Cookies, db: State<Database>) -> Json<JsonValue> {
    let username = get_username_from_cookie(&db, cookies.get(JWT_NAME)).unwrap();
    let (cn_type, cn_phonetics) = User::get_user_settings(&db, &username);
    let doc_titles: Vec<String> = UserDoc::aggregate_all_values_from_query(&db,
        doc!{"username": &username},
        vec!["title"]
    )[0].to_owned(); // unpacks
    let vocab_list = UserVocab::try_lookup_all(&db,
        doc!{"username": &username}
    ).unwrap();
    return Json(json!({
        "cn_type": cn_type,
        "cn_phonetics": cn_phonetics,
        "title_list": doc_titles, // Vec<String>
        "vocab_list": vocab_list, // Vec<Document>
    }));
}
/// /api/delete-doc/<doc_title>
#[get("/delete-user-doc/<doc_title>")]
pub fn delete_user_doc(cookies: Cookies, db: State<Database>, rt: State<Handle>, doc_title: &RawStr) -> Status {
    let title = convert_rawstr_to_string(doc_title);
    let username = get_username_from_cookie(&db, cookies.get(JWT_NAME)).unwrap();
    rt.block_on(UserDoc::try_delete(&db, &username, &title));
    return Status::Ok;
}
/// /api/delete-vocab/<vocab_uid>
#[get("/delete-user-vocab/<vocab_uid>")]
pub fn delete_user_vocab(cookies: Cookies, db: State<Database>, rt: State<Handle>, vocab_uid: &RawStr) -> Status {
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
#[derive(FromForm)]
pub struct UserAuthForm<'f> {
    username: &'f RawStr,
    password: &'f RawStr,
    email: &'f RawStr,
}
/// /api/login
#[post("/login", data = "<user_input>")]
pub fn login(mut cookies: Cookies, db: State<Database>, user_input: Form<UserAuthForm<'_>>) -> Status {
    let UserAuthForm { username, password, email } = user_input.into_inner();
    let username = convert_rawstr_to_string(username);
    let password = convert_rawstr_to_string(password);
    let email = convert_rawstr_to_string(email);
    let res_status = match User::check_password(&db, &username, &password) {
        true => {
            let new_cookie = generate_http_cookie(&db, username, password);
            cookies.add(new_cookie);
            Status::Accepted
        },
        false => Status::Unauthorized
    };
    return res_status;
}
/// /api/register
#[post("/register", data = "<user_input>")]
pub fn register(mut cookies: Cookies, db: State<Database>, user_input: Form<UserAuthForm<'_>>) -> Status {
    let UserAuthForm { username, password, email } = user_input.into_inner();
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
        Err(_) => Status::InternalServerError
    };
    return res_status;
}
#[derive(FromForm)]
pub struct UserDocForm<'f> {
    body: &'f RawStr,
    url: &'f RawStr,
    title: &'f RawStr,
    source: &'f RawStr,
}
/// /api/upload-user-doc
#[post("/upload-user-doc", data="<upload_doc>")]
pub fn upload_user_doc(cookies: Cookies, db: State<Database>, rt: State<Handle>, upload_doc: Form<UserDocForm<'_>>) -> Status {
    let UserDocForm { title, source, body, url } = upload_doc.into_inner();
    let title = convert_rawstr_to_string(title);
    let body = convert_rawstr_to_string(body);
    let source = convert_rawstr_to_string(source);
    let url = convert_rawstr_to_string(url);
    // Parse url if available
    let use_url = match url.as_str() {
        "" => false,
        _ => &body == ""
    };
    let res_status = match get_username_from_cookie(&db, cookies.get(JWT_NAME)) {
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
        None => Status::Unauthorized
    };
    return res_status;
}
#[derive(FromForm)]
pub struct SandboxDocForm<'f> {
    body: &'f RawStr,
    url: &'f RawStr,
    cn_type: &'f RawStr,
    cn_phonetics: &'f RawStr,
}
/// /api/upload-sandbox-doc
#[post("/upload-sandbox-doc", data="<upload_doc>")]
pub fn upload_sandbox_doc(db: State<Database>, rt: State<Handle>, upload_doc: Form<SandboxDocForm<'_>>) -> Json<JsonValue> {
    let SandboxDocForm { body, url, cn_type, cn_phonetics } = upload_doc.into_inner();
    let body = convert_rawstr_to_string(body);
    let url = convert_rawstr_to_string(url);
    let cn_type = convert_rawstr_to_string(cn_type);
    let cn_phonetics = convert_rawstr_to_string(cn_phonetics);
    // Parse url if available
    let try_url = match url.as_str() {
        "" => false,
        _ => &body == ""
    };
    let new_doc = match try_url {
        false => rt.block_on(SandboxDoc::new(body, cn_type, cn_phonetics, url)),
        true => rt.block_on(SandboxDoc::from_url(url, cn_type, cn_phonetics))
    };
    let res_json = match new_doc.try_insert(&db) {
        Ok(uid) => Json(json!({"uid": uid})),
        Err(e) => Json(json!({"error": e.to_string()}))
    };
    return res_json;
}
#[derive(FromForm)]
pub struct UserVocabForm<'f> {
    phrase_uid: &'f RawStr,
    from_doc_title: &'f RawStr,
}
/// /api/vocab
#[post("/upload-vocab", data="<user_vocab>")]
pub fn upload_vocab(cookies: Cookies, db: State<Database>, rt: State<Handle>, user_vocab: Form<UserVocabForm<'_>>) -> Status {
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
        None => Status::Unauthorized
    };
    return res_status;
}
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