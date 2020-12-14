#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;

use std::collections::HashMap;
use rocket::{
    request::Form,
    response::Redirect,
    http::RawStr,
    State,
};
use rocket_contrib::templates::Template;
use mongodb::sync::Database;
use ::duguo::*; // lib.rs

/* GET */
#[get("/")]
fn index() -> Template {
    let context: HashMap<&str, &str> = HashMap::new();
    return Template::render("index", context);
}

#[get("/login")]
fn login() -> Template {
    let context: HashMap<&str, &str> = HashMap::new();
    // TODO - check for JWT, and mention username in html if already logged-in
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
    let doc_id = convert_rawStr_to_String(doc_id);
    let html = match get_sandbox_document(db.clone(), doc_id) {
        Some(text) => {text},
        None => String::new()
    };
    if html != "" {
        context.insert("paragraph_html", &html);
    }
    return Template::render("reader", context);
}

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
fn login_form(db: State<Database>, user_input: Form<UserLoginForm<'_>>) -> Template {
    /*
    TODO - in comments
    */
    let UserLoginForm { username, password } = user_input.into_inner();
    let username = convert_rawStr_to_String(username);
    let password = convert_rawStr_to_String(password);

    let is_valid_password = check_password(db.clone(), username.clone(), password);
    let mut context = HashMap::new();
    if is_valid_password {
        // add login handling context!
        // (figure-out how to handle user "logged-in" status)
        return Template::render("index", context);
    }
    let password_incorrect_msg = format!("Incorrect password for {}. N incorrect attempts remaining for today", username);
    context.insert("message", password_incorrect_msg);
    // (record login attempt in database)
    return Template::render("login", context);
}

#[post("/login-post", data = "<user_input>")]
fn register_form(db: State<Database>, user_input: Form<UserRegisterForm<'_>>) -> Template {
    let UserRegisterForm { username, email, password } = user_input.into_inner();
    let username = convert_rawStr_to_String(username);
    let password = convert_rawStr_to_String(password);
    let email = convert_rawStr_to_String(email);

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
    let text_as_string = convert_rawStr_to_String(text);
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