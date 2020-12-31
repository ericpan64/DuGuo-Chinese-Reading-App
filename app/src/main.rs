#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;

use std::collections::HashMap;
use rocket::{
    request::Form,
    response::{Redirect},
    http::{RawStr, Cookie, Cookies, Status},
    State,
};
use rocket_contrib::templates::Template;
use mongodb::Database;
use tokio::runtime::{Runtime, Handle};
use ::duguo::*; // lib.rs

/* GET */
#[get("/")]
fn index(cookies: Cookies, db: State<Database>, rt: State<Handle>) -> Template {
    let mut context: HashMap<&str, String> = HashMap::new();
    let cookie_lookup = cookies.get(JWT_NAME);
    let username_from_cookie = rt.block_on(get_username_from_cookie(db.clone(), cookie_lookup));
    match username_from_cookie {
        Some(username) => { context.insert("username", username); },
        None =>  {}
    }
    return Template::render("index", context);
}

#[get("/login")]
fn login(cookies: Cookies, db: State<Database>, rt: State<Handle>) -> Template {
    let mut context: HashMap<&str, String> = HashMap::new();
    let cookie_lookup = cookies.get(JWT_NAME);
    let username_from_cookie = rt.block_on(get_username_from_cookie(db.clone(), cookie_lookup));
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
fn sandbox_view_doc(db: State<Database>, rt: State<Handle>, doc_id: &RawStr) -> Template {
    let mut context: HashMap<&str, &str> = HashMap::new();
    let doc_id = convert_rawstr_to_string(doc_id);
    let html = match rt.block_on(get_sandbox_document(db.clone(), doc_id)) {
        Some(text) => { rt.block_on(convert_string_to_tokenized_html(db.clone(), text)) },
        None => String::new()
    };
    if &html != "" {
        context.insert("paragraph_html", &html);
    }
    return Template::render("reader", context);
}

#[get("/u/<raw_username>")]
fn user_profile(cookies: Cookies, db: State<Database>, rt: State<Handle>, raw_username: &RawStr) -> Template {
    let mut context: HashMap<&str, String> = HashMap::new();
    let username = convert_rawstr_to_string(raw_username);
    match rt.block_on(check_if_username_exists(db.clone(), &username)) {
        true => { 
            context.insert("username", username.clone()); 
        },
        false => { }
    }
    // Compare username with logged-in username from JWT
    let cookie_lookup = cookies.get(JWT_NAME);
    match rt.block_on(get_username_from_cookie(db.clone(), cookie_lookup)) {
        Some(s) => { 
            context.insert("logged_in_username", s.clone()); 
            if &s == &username {
                let doc_html = rt.block_on(render_document_table(db.clone(), &username));
                let vocab_html = rt.block_on(render_vocab_table(db.clone(), &username));
            
                context.insert("doc_table", doc_html);
                context.insert("vocab_table", vocab_html);           
            }
        },
        None =>  { }
    }
    return Template::render("userprofile", context);
}

#[get("/u/<raw_username>/<doc_title>")]
fn user_view_doc(cookies: Cookies, db: State<Database>, rt: State<Handle>, raw_username: &RawStr, doc_title: &RawStr) -> Template {
    let mut context: HashMap<&str, String> = HashMap::new();
    let username = convert_rawstr_to_string(raw_username);
    match rt.block_on(check_if_username_exists(db.clone(), &username)) {
        true => { 
            context.insert("username", username.clone()); 
        },
        false => { }
    }
    // Compare username with logged-in username from JWT
    let cookie_lookup = cookies.get(JWT_NAME);
    match rt.block_on(get_username_from_cookie(db.clone(), cookie_lookup)) {
        Some(s) => { 
            if &s == &username {
                // Get html to render
                let title = convert_rawstr_to_string(doc_title);
                let doc_html = UserDoc::get_body_html_from_user_doc(db.clone(), &username, &title);
                let user_vocab_list_string = get_user_vocab_list_string(db.clone(), &username);

                let doc_res = rt.block_on(doc_html);
                context.insert("paragraph_html", doc_res.unwrap_or_default());
                let py_list_res = rt.block_on(user_vocab_list_string);
                context.insert("user_vocab_list_string", py_list_res.unwrap_or_default());
            }
        },
        None =>  {
            context.insert("paragraph_html", "<p>Not authenticated as user</p>".to_string());
        }
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
fn delete_user_doc(cookies: Cookies, db: State<Database>, rt: State<Handle>, doc_title: &RawStr) -> Redirect {
    let username_query = get_username_from_cookie(db.clone(), cookies.get(JWT_NAME));
    let title = convert_rawstr_to_string(doc_title);
    let username = rt.block_on(username_query).unwrap();
    rt.block_on(UserDoc::try_delete(db.clone(), &username, &title));
    return Redirect::to(uri!(user_profile: username));
}

#[get("/api/delete-vocab/<vocab_phrase>")]
fn delete_user_vocab(cookies: Cookies, db: State<Database>, rt: State<Handle>, vocab_phrase: &RawStr) -> Redirect {
    let phrase_string = convert_rawstr_to_string(vocab_phrase);
    let phrase_obj_creation = CnEnDictEntry::new(db.clone(), &phrase_string);
    let username_query = get_username_from_cookie(db.clone(), cookies.get(JWT_NAME));
    
    let username = rt.block_on(username_query).unwrap();
    let phrase_obj = rt.block_on(phrase_obj_creation);
    rt.block_on(UserVocab::try_delete(db.clone(), username.clone(), phrase_obj));
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
    saved_phrase: &'f RawStr,
    from_doc_title: &'f RawStr,
}

/* POST */
#[post("/sandbox/upload", data = "<user_text>")]
fn sandbox_upload(db: State<Database>, rt: State<Handle>, user_text: Form<TextForm<'_>>) -> Redirect {
    let TextForm { text } = user_text.into_inner();    
    let text_as_string = convert_rawstr_to_string(text);
    let new_doc = rt.block_on(SandboxDoc::new(db.clone(), text_as_string));
    let inserted_id = new_doc.try_insert(db.clone(), rt.clone()).unwrap();
    return Redirect::to(uri!(sandbox_view_doc: inserted_id));
}

#[post("/api/upload", data="<user_doc>")]
fn user_doc_upload(cookies: Cookies, db: State<Database>, rt: State<Handle>, user_doc: Form<UserDocumentForm<'_>>) -> Redirect {
    let UserDocumentForm { title, body } = user_doc.into_inner();
    let title = convert_rawstr_to_string(title);
    let body = convert_rawstr_to_string(body);

    let username_from_cookie = rt.block_on(get_username_from_cookie(db.clone(), cookies.get(JWT_NAME)));
    let res_redirect = match username_from_cookie {
        Some(username) => { 
            let new_doc = rt.block_on(UserDoc::new(db.clone(), username, title, body));
            match new_doc.try_insert(db.clone(), rt.clone()) {
                Ok(username) => { Redirect::to(uri!(user_profile: username)) },
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
fn user_vocab_upload(cookies: Cookies, db: State<Database>, rt: State<Handle>, user_vocab: Form<UserVocabForm<'_>>) -> Status {
    let UserVocabForm { saved_phrase, from_doc_title } = user_vocab.into_inner();
    let phrase = convert_rawstr_to_string(saved_phrase);
    println!("{}", &phrase); // TODO: confirm hanzi survives this...
    let from_doc_title = convert_rawstr_to_string(from_doc_title);

    let username_from_cookie = rt.block_on(get_username_from_cookie(db.clone(), cookies.get(JWT_NAME)));
    let res_status = match username_from_cookie {
        Some(username) => { 
            let new_doc = rt.block_on(UserVocab::new(db.clone(), username.clone(), phrase.clone(), from_doc_title));
            match new_doc.try_insert(db.clone(), rt.clone()) {
                Ok(_) => { Status::Accepted },
                Err(_) => { 
                    println!("Error when writing phrase {} for user {}", &phrase, &username); 
                    Status::BadRequest
                }
            }
        },
        None => {
            println!("Error: no username found from cookie");
            Status::BadRequest
        }
    };
    return res_status;
}

#[post("/login", data = "<user_input>")]
fn login_form(mut cookies: Cookies, db: State<Database>, rt: State<Handle>, user_input: Form<UserLoginForm<'_>>) -> Redirect {
    let UserLoginForm { username, password } = user_input.into_inner();
    let username = convert_rawstr_to_string(username);
    let password = convert_rawstr_to_string(password);

    let is_valid_password = rt.block_on(check_password(db.clone(), username.clone(), password.clone()));
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
#[post("/api/register", data = "<user_input>")]
fn register_form(mut cookies: Cookies, db: State<Database>, rt: State<Handle>, user_input: Form<UserRegisterForm<'_>>) -> Redirect {
    let UserRegisterForm { username, email, password } = user_input.into_inner();
    let username = convert_rawstr_to_string(username);
    let password = convert_rawstr_to_string(password);
    let email = convert_rawstr_to_string(email);

    let new_user = User::new(username.clone(), password.clone(), email);
    // TODO: figure-out way to handle registration error cases
    let res_redirect = match new_user.try_insert(db.clone(), rt.clone()) {
        Ok(_) => {
            let new_cookie = generate_http_cookie(username, password);
            cookies.add(new_cookie);
            Redirect::to(uri!(index))
        },
        Err(_) => { Redirect::to(uri!(login)) }
    };
    return res_redirect;
}

/* Server Startup */
fn main() -> Result<(), mongodb::error::Error>{
    let async_runtime = Runtime::new().unwrap();
    let rt = async_runtime.handle().clone(); // "Handle" is a clonable reference to the Runtime manager
    let db = connect_to_mongodb(rt.clone())?;

    rocket::ignite()
        .attach(Template::fairing())
        .manage(db)
        .manage(rt)
        .mount("/", routes![index, 
            login, login_form, register_form, 
            sandbox, sandbox_upload, sandbox_view_doc,
            user_profile, logout_user, 
            user_doc_upload, user_vocab_upload, user_view_doc,
            delete_user_doc, delete_user_vocab])
        .launch();

    return Ok(());
}