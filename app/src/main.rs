#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;

use std::collections::HashMap;
use rocket::{
    request::Form,
    response::{Redirect},
    http::{RawStr, Cookie, Cookies, Status},
    State,
};
use rocket_contrib::{
    templates::Template,
    serve::StaticFiles
};
use mongodb::sync::Database;
use tokio::runtime::{Runtime, Handle};

use ::duguo::*; // lib.rs
use ::duguo::{
    html_rendering::*,
    cookie_handling::*
};

/* GET */
#[get("/")]
fn index(cookies: Cookies, db: State<Database>) -> Template {
    let mut context: HashMap<&str, String> = HashMap::new();
    add_user_cookie_to_context(&cookies, &db, &mut context);
    return Template::render("index", context);
}

#[get("/login")]
fn login(cookies: Cookies, db: State<Database>) -> Template {
    let mut context: HashMap<&str, String> = HashMap::new();
    add_user_cookie_to_context(&cookies, &db, &mut context);
    return Template::render("login", context);
}

#[get("/sandbox")]
fn sandbox(cookies: Cookies, db: State<Database>) -> Template {
    let mut context: HashMap<&str, String> = HashMap::new();
    add_user_cookie_to_context(&cookies, &db, &mut context);
    return Template::render("sandbox", context);
}

#[get("/sandbox/<doc_id>")]
fn sandbox_view_doc(db: State<Database>, doc_id: &RawStr) -> Template {
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
fn feedback(cookies: Cookies, db: State<Database>) -> Template {
    let mut context: HashMap<&str, String> = HashMap::new();
    add_user_cookie_to_context(&cookies, &db, &mut context);
    return Template::render("feedback", context);
}

#[get("/u/<raw_username>")]
fn user_profile(cookies: Cookies, db: State<Database>, raw_username: &RawStr) -> Template {
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
fn user_view_doc(cookies: Cookies, db: State<Database>, raw_username: &RawStr, doc_title: &RawStr) -> Template {
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

#[get("/api/logout")]
fn logout_user(mut cookies: Cookies) -> Redirect {
    let mut removal_cookie = Cookie::named(JWT_NAME);
    removal_cookie.set_path("/");
    cookies.remove(removal_cookie);
    return Redirect::to(uri!(index));
}

#[get("/api/delete-doc/<doc_title>")]
fn delete_user_doc(cookies: Cookies, db: State<Database>, doc_title: &RawStr) -> Redirect {
    let title = convert_rawstr_to_string(doc_title);
    let username = get_username_from_cookie(&db, cookies.get(JWT_NAME)).unwrap();
    UserDoc::try_delete(&db, &username, &title);
    return Redirect::to(uri!(user_profile: username));
}

#[get("/api/delete-vocab/<vocab_phrase>")]
fn delete_user_vocab(cookies: Cookies, db: State<Database>, vocab_phrase: &RawStr) -> Redirect {
    let phrase_string = convert_rawstr_to_string(vocab_phrase);
    let username = get_username_from_cookie(&db, cookies.get(JWT_NAME)).unwrap();
    
    let (cn_type, _) = User::get_user_settings(&db, &username);
    UserVocab::try_delete(&db, &username, &phrase_string, &cn_type);
    return Redirect::to(uri!(user_profile: username));
}

// Note: these need to be defined here since they use Rocket macros
/* POST Forms */
#[derive(FromForm)]
struct UserLoginForm<'f> {
    username: &'f RawStr,
    password: &'f RawStr,
}

#[derive(FromForm)]
struct UserRegisterForm<'f> {
    username: &'f RawStr,
    email: &'f RawStr,
    password: &'f RawStr,
}

#[derive(FromForm)]
struct SandboxForm<'f> {
    text: &'f RawStr,
    cn_type: &'f RawStr,
    cn_phonetics: &'f RawStr,
}

#[derive(FromForm)]
struct SandboxUrlForm<'f> {
    url: &'f RawStr,
    cn_type: &'f RawStr,
    cn_phonetics: &'f RawStr,
}

#[derive(FromForm)]
struct UserDocumentForm<'f> {
    title: &'f RawStr,
    body: &'f RawStr,
}

#[derive(FromForm)]
struct UrlForm<'f> {
    url: &'f RawStr,
}

#[derive(FromForm)]
struct UserSettingForm<'f> {
    setting: &'f RawStr,
}

#[derive(FromForm)]
struct UserVocabForm<'f> {
    saved_phrase: &'f RawStr,
    from_doc_title: &'f RawStr,
}

#[derive(FromForm)]
struct UserFeedbackForm<'f> {
    feedback: &'f RawStr,
    contact: &'f RawStr,
}

/* POST */
#[post("/api/update-settings", data = "<user_setting>")]
fn update_settings(cookies: Cookies, db: State<Database>, user_setting: Form<UserSettingForm<'_>>) -> Status {
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

#[post("/api/sandbox-upload", data = "<user_text>")]
fn sandbox_upload(db: State<Database>, rt: State<Handle>, user_text: Form<SandboxForm<'_>>) -> Redirect {
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

#[post("/api/sandbox-url-upload", data = "<user_url>")]
fn sandbox_url_upload(db: State<Database>, rt: State<Handle>, user_url: Form<SandboxUrlForm<'_>>) -> Redirect {
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

#[post("/api/url-upload", data = "<user_url>")]
fn user_url_upload(cookies: Cookies, db: State<Database>, rt: State<Handle>, user_url: Form<UrlForm<'_>>) -> Redirect {
    let UrlForm { url } = user_url.into_inner();
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
                    Redirect::to(uri!(index)) 
                } 
            }
        },
        None => Redirect::to(uri!(index))
    };
    return res_redirect;
}

#[post("/api/upload", data="<user_doc>")]
fn user_doc_upload(cookies: Cookies, db: State<Database>, rt: State<Handle>, user_doc: Form<UserDocumentForm<'_>>) -> Redirect {
    let UserDocumentForm { title, body } = user_doc.into_inner();
    let title = convert_rawstr_to_string(title);
    let body = convert_rawstr_to_string(body);

    let username_from_cookie = get_username_from_cookie(&db, cookies.get(JWT_NAME));
    let res_redirect = match username_from_cookie {
        Some(username) => { 
            let new_doc = (rt).block_on(UserDoc::new(&db, username, title, body, None));
            match new_doc.try_insert(&db) {
                Ok(username) => Redirect::to(uri!(user_profile: username)),
                Err(e) => {
                    eprintln!("Exception when inserting doc: {:?}", e);
                    Redirect::to(uri!(index))
                }
            }
        },
        None => Redirect::to(uri!(index))
    };
    return res_redirect;
}

#[post("/api/vocab", data="<user_vocab>")]
fn user_vocab_upload(cookies: Cookies, db: State<Database>, rt: State<Handle>, user_vocab: Form<UserVocabForm<'_>>) -> Status {
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
fn login_form(mut cookies: Cookies, db: State<Database>, user_input: Form<UserLoginForm<'_>>) -> Status {
    let UserLoginForm { username, password } = user_input.into_inner();
    let username = convert_rawstr_to_string(username);
    let password = convert_rawstr_to_string(password);

    let is_valid_password = User::check_password(&db, &username, &password);
    let res_status = match is_valid_password {
        true => {
            let new_cookie = generate_http_cookie(username, password);
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
fn register_form(mut cookies: Cookies, db: State<Database>, user_input: Form<UserRegisterForm<'_>>) -> Status {
    let UserRegisterForm { username, email, password } = user_input.into_inner();
    let username = convert_rawstr_to_string(username);
    let password = convert_rawstr_to_string(password);
    let email = convert_rawstr_to_string(email);

    let new_user = User::new(username.clone(), password.clone(), email); // clone() makes sense here
    // TODO: figure-out way to handle registration error cases
    let res_status = match new_user.try_insert(&db) {
        Ok(_) => {
            let new_cookie = generate_http_cookie(username, password);
            cookies.add(new_cookie);
            Status::Accepted
        },
        Err(_) => { Status::UnprocessableEntity }
    };
    return res_status;
}

#[post("/api/feedback", data = "<user_feedback>")]
fn feedback_form(db: State<Database>, user_feedback: Form<UserFeedbackForm<'_>>) -> Redirect {
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

/* Server Startup */
fn main() -> Result<(), Box<dyn std::error::Error>>{
    let db = connect_to_mongodb()?;
    let rt = Runtime::new().unwrap();
    let rt = rt.handle().clone();
    
    rocket::ignite()
        .attach(Template::fairing())
        .manage(db)
        .manage(rt)
        .mount("/", routes![index, 
            login, login_form, register_form, 
            sandbox, sandbox_upload, sandbox_url_upload, sandbox_view_doc, 
            feedback, feedback_form,
            user_profile, logout_user, update_settings,
            user_doc_upload, user_url_upload, user_vocab_upload, user_view_doc,
            delete_user_doc, delete_user_vocab])
        .mount("/static", StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/static")))
        .launch();

    return Ok(());
}