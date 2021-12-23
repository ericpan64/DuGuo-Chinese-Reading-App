/*
/// API Route Handling
/// Expected form inputs are stored as Structs and defined above the corresponding route.
/// Note: the `/api/` base dir is appended when route is mounted in lib.rs.
*/

use crate::{
    convert_rawstr_to_string, 
    routes as Routes,
    DatabaseItem,
    auth::{generate_http_cookie, get_username_from_cookie},
    config::JWT_NAME,
    models::{
        public::{AppFeedback, SandboxDoc},
        user::{User, UserDoc, UserVocab, UserVocabList},
        zh::{CnType, CnPhonetics}
    }
};
use mongodb::{
    bson::{doc, document::Document},
    sync::Database
};
use rocket::{
    http::{RawStr, Cookie, Cookies, Status},
    request::Form,
    response::Redirect,
    State,
};
use rocket_contrib::{json, json::{Json, JsonValue}};
use tokio::runtime::Handle;

// /* GET */
/// /api/get-doc/<doc_id>
/// Attempts UserDoc first, otherwise SandboxDoc
#[get("/get-doc/<doc_id>")]
pub fn get_doc(cookies: Cookies, db: State<Database>, doc_id: &RawStr) -> Json<JsonValue> {
    let doc_id = convert_rawstr_to_string(doc_id);
    let mut user_doc_query: Option<Document> = None;
    if let Some(username) = get_username_from_cookie(&db, cookies.get(JWT_NAME)) {
        user_doc_query = UserDoc::try_lookup_one(&db, doc!{"username": username, "doc_id": &doc_id});
    }
    let res: Document = match user_doc_query {
        Some(doc) => doc,
        None => {
            match SandboxDoc::try_lookup_one(&db, doc!{"doc_id": &doc_id}) {
                Some(doc) => doc,
                None => doc!{"error": "No document found"}
            }
        }
    };
    return Json(json!{res});
}
/// /api/get-all-user-items
#[get("/get-all-user-items")]
pub fn get_all_user_items(cookies: Cookies, db: State<Database>) -> Json<JsonValue> {
    let username = get_username_from_cookie(&db, cookies.get(JWT_NAME)).unwrap();
    let (cn_type, cn_phonetics) = User::get_user_settings(&db, &username);
    let doc_list = UserDoc::try_lookup_all(&db,
        doc!{"username": &username},
    ).unwrap();
    let vocab_list = UserVocab::try_lookup_all(&db,
        doc!{"username": &username}
    ).unwrap();
    return Json(json!({
        "cn_type": cn_type,
        "cn_phonetics": cn_phonetics,
        "doc_list": doc_list, // Vec<Document>
        "vocab_list": vocab_list, // Vec<Document>
    }));
}
/// /api/get-user-vocab-string
#[get("/get-user-vocab-string")]
pub fn get_user_vocab_string(cookies: Cookies, db: State<Database>) -> Json<JsonValue> {
    let username = get_username_from_cookie(&db, cookies.get(JWT_NAME)).unwrap();
    let (cn_type, _) = User::get_user_settings(&db, &username);
    let res = UserVocabList::try_lookup_one(&db, 
        doc!{"username": username, "cn_type": cn_type.as_str()}
    ).unwrap();
    return Json(json!({
        "res": res
    }));
}
/// /api/delete-user-doc/<doc_title>
#[get("/delete-user-doc/<doc_title>")]
pub fn delete_user_doc(cookies: Cookies, db: State<Database>, rt: State<Handle>, doc_title: &RawStr) -> Status {
    let title = convert_rawstr_to_string(doc_title);
    let username = get_username_from_cookie(&db, cookies.get(JWT_NAME)).unwrap();
    rt.block_on(UserDoc::try_delete(&db, &username, &title));
    return Status::Ok;
}
/// /api/delete-user-vocab/<vocab_uid>
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
pub fn logout(mut cookies: Cookies) -> Redirect {
    let mut removal_cookie = Cookie::named(JWT_NAME);
    removal_cookie.set_path("/");
    cookies.remove(removal_cookie);
    return Redirect::to("/");
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
pub fn feedback(db: State<Database>, user_feedback: Form<AppFeedbackForm<'_>>) -> Redirect {
    let AppFeedbackForm { feedback, contact } = user_feedback.into_inner();
    let feedback = convert_rawstr_to_string(feedback);
    let contact = convert_rawstr_to_string(contact);
    let new_feedback = AppFeedback::new(feedback, contact);
    new_feedback.try_insert(&db).unwrap();
    return Redirect::to(uri!(Routes::feedback));
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
    let _ = email;
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
pub fn upload_user_doc(cookies: Cookies, db: State<Database>, rt: State<Handle>, upload_doc: Form<UserDocForm<'_>>) -> Redirect {
    let UserDocForm { title, source, body, url } = upload_doc.into_inner();
    let desired_title = convert_rawstr_to_string(title);
    let body = convert_rawstr_to_string(body);
    let source = convert_rawstr_to_string(source);
    let url = convert_rawstr_to_string(url);
    let res_status = match get_username_from_cookie(&db, cookies.get(JWT_NAME)) {
        Some(username) => { 
            let new_doc = match url.as_str() != "" {
                true => rt.block_on(UserDoc::from_url(&db, username.clone(), url)),
                false => rt.block_on(UserDoc::new(&db, username.clone(), desired_title, body, source))
            };
            match new_doc.try_insert(&db) {
                Ok(doc_title) => Redirect::to(uri!(Routes::user_doc: &username, doc_title)),
                Err(e) => {
                    eprintln!("Exception when inserting doc: {:?}", e);
                    Redirect::to(uri!(Routes::user_profile: &username))
                }
            }
        },
        None => Redirect::to("/")
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
pub fn upload_sandbox_doc(db: State<Database>, rt: State<Handle>, upload_doc: Form<SandboxDocForm<'_>>) -> Redirect {
    let SandboxDocForm { body, url, cn_type, cn_phonetics } = upload_doc.into_inner();
    let body = convert_rawstr_to_string(body);
    let url = convert_rawstr_to_string(url);
    let cn_type = convert_rawstr_to_string(cn_type);
    let cn_phonetics = convert_rawstr_to_string(cn_phonetics);
    let new_doc = match url.as_str() != "" {
        true => rt.block_on(SandboxDoc::from_url(url, cn_type, cn_phonetics)),
        false => rt.block_on(SandboxDoc::new(body, cn_type, cn_phonetics, url))
    };
    let doc_id = new_doc.try_insert(&db).unwrap();
    return Redirect::to(uri!(Routes::sandbox_doc: doc_id));
}
#[derive(FromForm)]
pub struct UserVocabForm<'f> {
    phrase_uid: &'f RawStr,
    from_doc_title: &'f RawStr,
}
/// /api/upload-vocab
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