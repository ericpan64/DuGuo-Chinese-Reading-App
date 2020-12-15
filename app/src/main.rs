#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;

use std::collections::HashMap;
use rocket::{
    request::Form,
    response::Redirect,
    http::{RawStr, Cookies},
    State,
};
use rocket_contrib::templates::Template;
use mongodb::sync::Database;
use ::duguo::*; // lib.rs

/* GET */
#[get("/")]
fn index(cookies: Cookies, db: State<Database>) -> Template {
    let mut context: HashMap<&str, String> = HashMap::new();
    let cookie_lookup = cookies.get(JWT_NAME);
    let username_from_cookie = get_username_from_cookie(db.clone(), cookie_lookup);
    match username_from_cookie {
        Some(username) => { context.insert("username", username); },
        None =>  {}
    }
    return Template::render("index", context);
}

#[get("/login")]
fn login(cookies: Cookies, db: State<Database>) -> Template {
    let mut context: HashMap<&str, String> = HashMap::new();
    let cookie_lookup = cookies.get(JWT_NAME);
    let username_from_cookie = get_username_from_cookie(db.clone(), cookie_lookup);
    match username_from_cookie {
        Some(username) => { context.insert("username", username); },
        None =>  {}
    }
    return Template::render("login", context);
}

#[get("/sandbox")]
fn sandbox() -> Template {
    let context: HashMap<&str, &str> = HashMap::new();
    return Template::render("sandbox", context);
}

#[get("/sandbox/<doc_id>")]
fn sandbox_view_doc(db: State<Database>, doc_id: &RawStr) -> Template {
    /*
    TODO - Figure-out how to render interactive text
    */
    let mut context: HashMap<&str, &str> = HashMap::new();
    // Note: var html will _eventually_ need to be html, though plaintext for now
    let doc_id = convert_rawstr_to_string(doc_id);
    let html = match get_sandbox_document(db.clone(), doc_id) {
        Some(text) => {text},
        None => String::new()
    };
    if html != "" {
        context.insert("paragraph_html", &html);
    }
    return Template::render("reader", context);
}

// Note: these need to be here since they use Rocket macros
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
struct TextForm<'f> {
    text: &'f RawStr,
}

/* POST */
#[post("/login", data = "<user_input>")]
fn login_form(mut cookies: Cookies, db: State<Database>, user_input: Form<UserLoginForm<'_>>) -> Redirect {
    /*
    TODO - in comments
    */
    let UserLoginForm { username, password } = user_input.into_inner();
    let username = convert_rawstr_to_string(username);
    let password = convert_rawstr_to_string(password);

    let is_valid_password = check_password(db.clone(), username.clone(), password.clone());
    let mut context = HashMap::new();
    let res_redirect = match is_valid_password {
        true => {
            let new_cookie = generate_http_cookie(username, password);
            cookies.add(new_cookie);
            Redirect::to(uri!(index))
        },
        false => {
            // (record login attempt in database)
            let password_incorrect_msg = format!("Incorrect password for {}. N incorrect attempts remaining for today", username);
            context.insert("message", password_incorrect_msg);
            Redirect::to(uri!(login))
        }
    };
    return res_redirect;
}

#[post("/login-post", data = "<user_input>")]
fn register_form(db: State<Database>, user_input: Form<UserRegisterForm<'_>>) -> Template {
    let UserRegisterForm { username, email, password } = user_input.into_inner();
    let username = convert_rawstr_to_string(username);
    let password = convert_rawstr_to_string(password);
    let email = convert_rawstr_to_string(email);

    let new_user = User::new(username, password, email);
    let message = new_user.try_insert(db.clone()).unwrap();

    let mut context: HashMap<&str, &str> = HashMap::new();
    context.insert("message", message.as_str());
    return Template::render("login", context);
}

#[post("/sandbox/upload", data = "<user_text>")]
fn sandbox_upload(db: State<Database>, user_text: Form<TextForm<'_>>) -> Redirect {
    /* 
    TODO - in comments
    */
    let TextForm { text } = user_text.into_inner();    
    let text_as_string = convert_rawstr_to_string(text);
    let new_doc = SandboxDoc::new(text_as_string);
    let inserted_id = new_doc.try_insert(db.clone()).unwrap();
    // Redirect to URL with document ID
    return Redirect::to(uri!(sandbox_view_doc: inserted_id));
}

/* Server Startup */
fn main() -> Result<(), mongodb::error::Error>{
    let db = init_mongodb()?;

    rocket::ignite()
        .attach(Template::fairing())
        .manage(db)
        .mount("/", routes![index, 
            login, login_form, register_form, 
            sandbox, sandbox_upload, sandbox_view_doc])
        .launch();

    return Ok(());
}