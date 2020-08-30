#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;


use std::collections::HashMap;
use std::path::Path;
use std::fs::File;

use rocket::request::Form;
use rocket::http::RawStr;
use rocket::Data;
use rocket_contrib::templates::Template;

use mongodb::options::ClientOptions;
use mongodb::options::StreamAddress;

use uuid::Uuid;

#[get("/")]
fn index() -> Template {
    let context: HashMap<&str, &str> = HashMap::new();
    Template::render("index", context)
}

#[get("/login")]
fn login() -> Template {
    let context: HashMap<&str, &str> = HashMap::new();
    Template::render("login", context)
}

#[derive(FromForm)]
struct UserLogin<'f> {
    user: &'f RawStr,
    pass: &'f RawStr,
    checkbox: bool,
}

#[derive(FromForm)]
struct UserRegister<'f> {
    user: &'f RawStr,
    email: &'f RawStr,
    pass: &'f RawStr,
    checkbox: bool,
}

#[post("/login", data = "<user_input>")]
fn login_form(user_input: Form<UserLogin<'_>>) -> String {
    format!("Returned user: {}\n pass: {}\n checkbox: {}", 
            user_input.user, user_input.pass, user_input.checkbox)

}

#[post("/register", data = "<user_input>")]
fn register_form(user_input: Form<UserRegister<'_>>) -> String {
    format!("Returned user: {}\n email:{} \n pass: {}\n checkbox: {}", 
        user_input.user, user_input.email, user_input.pass, user_input.checkbox)
}

#[get("/sandbox")]
fn sandbox() -> Template {
    let context: HashMap<&str, &str> = HashMap::new();
    Template::render("sandbox", context)
}


#[post("/sandbox/upload", data = "<user_file>")]
fn sandbox_upload(user_file: Data) -> Result<String, std::io::Error> {
    // get file from POST request
    // create temp document
    let id = Uuid::new_v4();
    let filename = format!("temp/{}", id);
    let url = format!("{host}/sandbox/{id}\n", host="http://localhost:8000", id = id);

    user_file.stream_to_file(Path::new(&filename))?;
    //  if successful: route to view
    //  else: show error

    //placeholder
    Ok(url)
}

#[get("/sandbox/<doc_id>")]
fn sandbox_view_doc(doc_id: &RawStr) -> Option<File> {
    // placeholder
    let filename = format!("temp/{}", doc_id);
    File::open(&filename).ok()

}


fn main() -> Result<(), mongodb::error::Error>{
    // init mongodb
    let options = ClientOptions::builder()
    .hosts(vec![
        StreamAddress {
            hostname: "localhost".into(),
            port: Some(27017),
        }
    ])
    .build();

    let client = mongodb::Client::with_options(options)?;

    // init rocket
    rocket::ignite()
        .attach(Template::fairing())
        .mount("/", routes![index, 
            login, login_form, register_form, 
            sandbox, sandbox_upload, sandbox_view_doc])
        .launch();

    Ok(())
}


// Refactor 1 Template code


#[get("/upload")]
fn upload() -> () {

}

#[get("/u/<username>")]
fn view_user(username: String) -> () {

}
