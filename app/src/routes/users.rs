/*
/// Route handling for authenticated users
/// 
/// users.rs
/// ├── GET
/// |   └── /u/<username>
/// |   └── /u/<username>/<doc_title>
/// |   └── /api/delete-doc/<doc_title>
/// |   └── /api/delete-vocab/<phrase>
/// |   └── /api/logout
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
    html::{render_document_table, render_vocab_table},
    models::{
        user::{User, UserDoc, UserVocab, UserVocabList},
        zh::{CnType, CnPhonetics}
    }
};
use mongodb::sync::Database;
use rocket::{
    http::{RawStr, Cookie, Cookies, Status},
    request::Form,
    response::Redirect,
    State,
};
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use tokio::runtime::Handle;

/* GET */
#[get("/u/<raw_username>")]
pub fn user_profile(cookies: Cookies, db: State<Database>, raw_username: &RawStr) -> Template {
    let mut context: HashMap<&str, String> = HashMap::new(); // Note: <&str, String> makes more sense than <&str, &str> due to variable lifetimes
    let username = convert_rawstr_to_string(raw_username);
    // Compare username with logged-in username from JWT
    match get_username_from_cookie(&db, cookies.get(JWT_NAME)) {
        Some(s) => { 
            if &s == &username {
                let (cn_type, cn_phonetics) = User::get_user_settings(&db, &username);
                let doc_html = render_document_table(&db, &username);
                let vocab_html = render_vocab_table(&db, &username);
            
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
    return Template::render("userprofile", context);
}


#[get("/u/<raw_username>/<doc_title>")]
pub fn user_view_doc(cookies: Cookies, db: State<Database>, raw_username: &RawStr, doc_title: &RawStr) -> Template {
    let mut context: HashMap<&str, String> = HashMap::new(); // Note: <&str, String> makes more sense than <&str, &str> due to variable lifetimes
    let username = convert_rawstr_to_string(raw_username);
    // Compare username with logged-in username from JWT
    match get_username_from_cookie(&db, cookies.get(JWT_NAME)) {
        Some(s) => { 
            if &s == &username {
                // Get html to render
                let (_, cn_phonetics) = User::get_user_settings(&db, &username);
                let title = convert_rawstr_to_string(doc_title);
                let doc_html_res = UserDoc::get_body_html_from_user_doc(&db, &username, &title).unwrap_or_default();
                let user_vocab_list_string_res = UserVocabList::get_user_vocab_list_string(&db, &username).unwrap_or_default();
                context.insert("paragraph_html", doc_html_res);
                context.insert("user_vocab_list_string", user_vocab_list_string_res);
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

#[get("/api/delete-doc/<doc_title>")]
pub fn delete_user_doc(cookies: Cookies, db: State<Database>, doc_title: &RawStr) -> Redirect {
    let title = convert_rawstr_to_string(doc_title);
    let username = get_username_from_cookie(&db, cookies.get(JWT_NAME)).unwrap();
    UserDoc::try_delete(&db, &username, &title);
    return Redirect::to(uri!(user_profile: username));
}

#[get("/api/delete-vocab/<vocab_phrase>")]
pub fn delete_user_vocab(cookies: Cookies, db: State<Database>, vocab_phrase: &RawStr) -> Redirect {
    let phrase_string = convert_rawstr_to_string(vocab_phrase);
    let username = get_username_from_cookie(&db, cookies.get(JWT_NAME)).unwrap();
    
    let (cn_type, _) = User::get_user_settings(&db, &username);
    UserVocab::try_delete(&db, &username, &phrase_string, &cn_type);
    return Redirect::to(uri!(user_profile: username));
}

#[get("/api/logout")]
pub fn logout_user(mut cookies: Cookies) -> Redirect {
    let mut removal_cookie = Cookie::named(JWT_NAME);
    removal_cookie.set_path("/");
    cookies.remove(removal_cookie);
    return Redirect::to("/");
}

#[derive(FromForm)]
pub struct UserLoginForm<'f> {
    username: &'f RawStr,
    password: &'f RawStr,
}

#[derive(FromForm)]
pub struct UserRegisterForm<'f> {
    username: &'f RawStr,
    email: &'f RawStr,
    password: &'f RawStr,
}


#[derive(FromForm)]
pub struct UserUrlForm<'f> {
    url: &'f RawStr,
}


#[derive(FromForm)]
pub struct UserVocabForm<'f> {
    saved_phrase: &'f RawStr,
    from_doc_title: &'f RawStr,
}

#[derive(FromForm)]
pub struct UserSettingForm<'f> {
    setting: &'f RawStr,
}

#[post("/api/update-settings", data = "<user_setting>")]
pub fn update_settings(cookies: Cookies, db: State<Database>, user_setting: Form<UserSettingForm<'_>>) -> Status {
    let UserSettingForm { setting } = user_setting.into_inner();
    let setting = convert_rawstr_to_string(setting);
    let username_from_cookie = get_username_from_cookie(&db, cookies.get(JWT_NAME));
    let res_status = match username_from_cookie {
        Some(username) => {
            let cn_type = match setting.as_str() {
                "trad" => Some(CnType::Traditional),
                "simp" => Some(CnType::Simplified),
                _ => None,
            };
            let cn_phonetics = match setting.as_str() {
                "pinyin" => Some(CnPhonetics::Pinyin),
                "zhuyin" => Some(CnPhonetics::Zhuyin),
                _ => None,
            };
            match User::update_user_settings(&db, &username, cn_type, cn_phonetics) {
                Ok(_) => Status::Accepted,
                Err(_) => Status::BadRequest
            }
        },
        None => Status::Unauthorized
    };
    return res_status;
}

#[post("/api/url-upload", data = "<user_url>")]
pub fn user_url_upload(cookies: Cookies, db: State<Database>, rt: State<Handle>, user_url: Form<UserUrlForm<'_>>) -> Redirect {
    let UserUrlForm { url } = user_url.into_inner();
    let url = convert_rawstr_to_string(url); // Note: ':' is removed
    // read http header if present
    let url = url.replace("http//", "http://");
    let url = url.replace("https//", "https://");
    let username_from_cookie = get_username_from_cookie(&db, cookies.get(JWT_NAME));
    let res_redirect = match username_from_cookie {
        Some(username) => { 
            let new_doc = (rt).block_on(UserDoc::from_url(&db, username, url));
            match new_doc.try_insert(&db) {
                Ok(username) => Redirect::to(uri!(user_profile: username)),
                Err(e) => { 
                    eprintln!("Exception when inserting doc from url: {:?}", e);
                    Redirect::to("/") 
                } 
            }
        },
        None => Redirect::to("/")
    };
    return res_redirect;
}

#[derive(FromForm)]
pub struct UserDocumentForm<'f> {
    title: &'f RawStr,
    source: &'f RawStr,
    body: &'f RawStr,
}

#[post("/api/upload", data="<user_doc>")]
pub fn user_doc_upload(cookies: Cookies, db: State<Database>, rt: State<Handle>, user_doc: Form<UserDocumentForm<'_>>) -> Redirect {
    let UserDocumentForm { title, source, body } = user_doc.into_inner();
    let title = convert_rawstr_to_string(title);
    let body = convert_rawstr_to_string(body);
    let source = convert_rawstr_to_string(source);
    // read http header if present
    let source = source.replace("http//", "http://");
    let source = source.replace("https//", "https://");
    
    let username_from_cookie = get_username_from_cookie(&db, cookies.get(JWT_NAME));
    let res_redirect = match username_from_cookie {
        Some(username) => { 
            let new_doc = (rt).block_on(UserDoc::new(&db, username, title, body, source));
            match new_doc.try_insert(&db) {
                Ok(username) => Redirect::to(uri!(user_profile: username)),
                Err(e) => {
                    eprintln!("Exception when inserting doc: {:?}", e);
                    Redirect::to("/")
                }
            }
        },
        None => Redirect::to("/")
    };
    return res_redirect;
}

#[post("/api/vocab", data="<user_vocab>")]
pub fn user_vocab_upload(cookies: Cookies, db: State<Database>, rt: State<Handle>, user_vocab: Form<UserVocabForm<'_>>) -> Status {
    let UserVocabForm { saved_phrase, from_doc_title } = user_vocab.into_inner();
    let phrase = convert_rawstr_to_string(saved_phrase);
    let from_doc_title = convert_rawstr_to_string(from_doc_title);

    let username_from_cookie = get_username_from_cookie(&db, cookies.get(JWT_NAME));
    let res_status = match username_from_cookie {
        Some(username) => { 
            let new_vocab = (rt).block_on(UserVocab::new(&db, username, phrase, from_doc_title));
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


#[post("/api/login", data = "<user_input>")]
pub fn login_form(mut cookies: Cookies, db: State<Database>, user_input: Form<UserLoginForm<'_>>) -> Status {
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
        false => {
            // (TODO: record login attempt in database, limit 8 per day)
            Status::Unauthorized
        }
    };
    return res_status;
}

#[post("/api/register", data = "<user_input>")]
pub fn register_form(mut cookies: Cookies, db: State<Database>, user_input: Form<UserRegisterForm<'_>>) -> Status {
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
        Err(_) => { Status::UnprocessableEntity }
    };
    return res_status;
}
