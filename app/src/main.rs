#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;

use std::collections::HashMap;
use rocket::{
    request::Form,
    response::Redirect,
    http::{RawStr, Cookie, Cookies},
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
    // TODO: update get_sandbox_document to return an HtmlString (it's just plaintext for now)
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

#[get("/u/<raw_username>")]
fn user_profile(cookies: Cookies, db: State<Database>, raw_username: &RawStr) -> Template {
    let mut context: HashMap<&str, String> = HashMap::new();
    let username = convert_rawstr_to_string(raw_username);
    match check_if_username_exists(db.clone(), &username) {
        true => { 
            context.insert("username", username.clone()); 
        },
        false => { }
    }
    // lookup username in db
    // get logged-in username from JWT
    let cookie_lookup = cookies.get(JWT_NAME);
    match get_username_from_cookie(db.clone(), cookie_lookup) {
        Some(s) => { 
            context.insert("logged_in_username", s.clone()); 
            if &s == &username {
                let HtmlString(doc_html) = render_document_table(db.clone(), &username);
                let HtmlString(vocab_html) = render_vocab_table(db.clone(), &username);
            
                context.insert("doc_table", doc_html);
                context.insert("vocab_table", vocab_html);           
            }
        },
        None =>  { }
    }
    return Template::render("userprofile", context);
}

#[get("/api/logout")]
fn logout_user(mut cookies: Cookies) -> Redirect {
    let mut removal_cookie = Cookie::named(JWT_NAME);
    removal_cookie.set_path("/");
    cookies.remove(removal_cookie);
    return Redirect::to(uri!(index));
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

#[derive(FromForm)]
struct UserDocumentForm<'f> {
    title: &'f RawStr,
    body: &'f RawStr,
}

#[derive(FromForm)]
struct UserVocabForm<'f> {
    phrase: &'f RawStr,
    from_doc_title: &'f RawStr,
}

/* POST */
#[post("/api/upload", data="<user_doc>")]
fn user_doc_upload(cookies: Cookies, db: State<Database>, user_doc: Form<UserDocumentForm>) -> Redirect {
    let UserDocumentForm { title, body } = user_doc.into_inner();
    let title = convert_rawstr_to_string(title);
    let body = convert_rawstr_to_string(body);

    let username_from_cookie = get_username_from_cookie(db.clone(), cookies.get(JWT_NAME));
    let res_redirect = match username_from_cookie {
        Some(username) => { 
            let new_doc = UserDoc::new(username, title, body);
            match new_doc.try_insert(db.clone()) {
                Ok(username) => {Redirect::to(uri!(user_profile: username))},
                Err(_) => { Redirect::to(uri!(index)) } 
            }
        },
        None => {
            Redirect::to(uri!(index))
        }
    };
    return res_redirect;
}

#[post("/api/vocab", data="<user_vocab>")]
fn user_vocab_upload(cookies: Cookies, db: State<Database>, user_vocab: Form<UserVocabForm>) -> Redirect {
    // TODO convert this to be 
    let UserVocabForm { phrase, from_doc_title } = user_vocab.into_inner();
    let phrase = convert_rawstr_to_string(phrase);
    let from_doc_title = convert_rawstr_to_string(from_doc_title);

    let username_from_cookie = get_username_from_cookie(db.clone(), cookies.get(JWT_NAME));
    let res_redirect = match username_from_cookie {
        Some(username) => { 
            let new_doc = UserVocab::new(db.clone(), username, phrase, from_doc_title);
            match new_doc.try_insert(db.clone()) {
                Ok(username) => {Redirect::to(uri!(user_profile: username))},
                Err(_) => { Redirect::to(uri!(index)) } 
            }
        },
        None => {
            Redirect::to(uri!(index))
        }
    };
    return res_redirect;

}

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
            // TODO: remove message context, figure-out better way to display info
            let password_incorrect_msg = format!("Incorrect password for {}. N incorrect attempts remaining for today", username);
            context.insert("message", password_incorrect_msg);
            Redirect::to(uri!(login))
        }
    };
    return res_redirect;
}

// TODO: Change message handling to something neater, then update this to redirect instead of render
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
    load_cedict(db.clone())?; // TODO: test speedup with specified capacity
    rocket::ignite()
        .attach(Template::fairing())
        .manage(db)
        .mount("/", routes![index, 
            login, login_form, register_form, 
            sandbox, sandbox_upload, sandbox_view_doc,
            user_profile, logout_user, 
            user_doc_upload, user_vocab_upload])
        .launch();

    return Ok(());
}