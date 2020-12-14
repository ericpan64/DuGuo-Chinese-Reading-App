mod config;
use config::{
    DATABASE_NAME,
    USER_COLL_NAME,
    SANDBOX_COLL_NAME,
    str_to_hashed_string,
};
use mongodb::{
    bson::{doc, Bson, document::Document, to_document},
    options::{ClientOptions, StreamAddress},
    sync::{Client, Collection, Database},
    error::Error,
};
use rocket::{
    http::RawStr,
};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/* Structs */
// _ Database _
#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    username: String,
    pw_hash: String,
    email: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SandboxDoc {
    doc_id: String,
    body: String,
}

/* Traits */
pub trait DatabaseItem {
    fn as_document(&self) -> Document where Self: Serialize {
        return to_document(self).unwrap();
    }
    fn collection_name(&self) -> &str;
    fn try_insert(&self, db: Database) -> Result<String, Error>;
    fn is_saved_to_db(&self, db: Database) -> bool where Self: Serialize {
        let query_doc = self.as_document();
        let coll = db.collection(self.collection_name());
        let res = match coll.find_one(query_doc, None).unwrap() {
            Some(_) => true,
            None => false
        };
        return res;
    }
}

/* Struct Functionality */
impl User {
    pub fn new(username: String, password: String, email: String) -> Self {
        let pw_hash = str_to_hashed_string(password.as_str());
        let new_user = User {
            username,
            pw_hash,
            email,
        };
        return new_user;
    }
}

impl DatabaseItem for User {
    fn collection_name(&self) -> &str {
        return USER_COLL_NAME;
    }

    fn try_insert(&self, db: Database) -> Result<String, Error> {
        let is_new_username = check_existing_username(db.clone(), self.username.as_str());
        let is_new_email = check_existing_email(db.clone(), self.email.as_str());
        let can_register = is_new_username && is_new_email;
        let mut message = String::new();
        if can_register {
            let user_coll = db.collection(USER_COLL_NAME);
            let new_doc = self.as_document();
            match insert_one_doc(user_coll, new_doc) {
                Ok(_) => {
                    let success_msg = format!("Registration successful! Username: {}", self.username);
                    message.push_str(&success_msg);
                }
                Err(e) => {
                    let error_msg = format!("Error when pushing to database, log: {:?}", e);
                    message.push_str(&error_msg);
                }
            }
        } else {
            if !is_new_username {
                let user_taken_msg = format!("Username {} is already in-use. ", self.username);
                message.push_str(&user_taken_msg);
            }
            if !is_new_email {
                let email_taken_msg = format!("Email {} is already in-use. ", self.email);
                message.push_str(&email_taken_msg);
            }
        }
        return Ok(message);
    }
}

impl SandboxDoc {
    pub fn new(body: String) -> Self {
        let doc_id = Uuid::new_v4().to_string();
        let new_doc = SandboxDoc {
            doc_id,
            body,
        };
        return new_doc;
    }
}

impl DatabaseItem for SandboxDoc {
    fn collection_name(&self) -> &str {
        return SANDBOX_COLL_NAME;
    }

    fn try_insert(&self, db: Database) -> Result<String, Error> {
        let sandbox_coll = db.collection(SANDBOX_COLL_NAME);
        let new_doc = self.as_document();
        match insert_one_doc(sandbox_coll, new_doc) {
            Ok(_) => {}
            Err(e) => { return Err(e); }
        }
        return Ok(self.doc_id.clone());
    }
}

/* Other Public Functions */
pub fn init_mongodb() -> Result<Database, Error> {
    let options = ClientOptions::builder()
    .hosts(vec![
        StreamAddress {
            hostname: "localhost".into(),
            port: Some(27017),
        }
    ])
    .build();

    let client = Client::with_options(options)?;
    let db: Database = client.database(DATABASE_NAME);
    return Ok(db);
}

/// Returns "" if UTF-8 error is encountered
pub fn convert_rawStr_to_String(s: &RawStr) -> String {
    let res = match s.url_decode() {
        Ok(val) => val,
        Err(e) => {
            println!("UTF-8 Error: {:?}", e);
            String::new()
        }
    };
    return res;
}

pub fn get_sandbox_document(db: Database, doc_id: String) -> Option<String> {
    let query_doc = doc! { "doc_id": doc_id };
    let coll = db.collection(SANDBOX_COLL_NAME);
    let res = match coll.find_one(query_doc, None).unwrap() {
        Some(doc) => Some(doc.get("body").and_then(Bson::as_str).expect("No body was stored").to_string()),
        None => None
    };
    return res;
}

pub fn send_password_reset(email: String) -> bool {
    /* TODO: add functionality to generate+send reset email */
    /* Alternative email workflow. Send link */
    // create reset token
    // set expiration date
    // send email with reset link
    false
}

pub fn update_password(db: Database, user_to_check: User, new_pw: String) -> bool {
    let coll = db.collection(USER_COLL_NAME);
    let query_doc = to_document(&user_to_check).expect("Schema error, ya doofus!");
    let updated_user = User { pw_hash: str_to_hashed_string(new_pw.as_str()), ..user_to_check };
    let updated_doc = to_document(&updated_user).expect("Schema error :-/");
    let res = match coll.update_one(query_doc, updated_doc, None) {
        Ok(_) => true,
        Err(_) => false
    };
    return res;
}

pub fn check_password(db: Database, username: String, pw_to_check: String) -> bool {
    let coll = db.collection(USER_COLL_NAME);
    let hashed_pw = str_to_hashed_string(pw_to_check.as_str());
    let query_doc = doc! { "username": username, "pw_hash": hashed_pw.clone() };
    let res = match coll.find_one(query_doc, None).unwrap() {
        Some(document) => {
            let saved_hash = document.get("pw_hash").and_then(Bson::as_str).expect("No password was stored");
            saved_hash == hashed_pw
        }
        None => false
    };
    return res;
}


/* Private Functions */
fn insert_one_doc(coll: Collection, doc: Document) -> Result<(), Error> {
    coll.delete_one(doc.clone(), None)?; // remove once can set unique indices
    match coll.insert_one(doc.clone(), None) {
        Ok(_) => {}
        Err(e) => {
            println!("Skipping insert for doc: {:?}\n\tGot error: {:?}", doc, e);
        }
    }
    return Ok(());
}

fn check_existing_username(db: Database, username: &str) -> bool {
    let coll = db.collection(USER_COLL_NAME);
    let username_search = coll.find_one(doc! { "username": username }, None).unwrap();
    return username_search == None;
}

fn check_existing_email(db: Database, email: &str) -> bool {
    let coll = db.collection(USER_COLL_NAME);
    let email_search = coll.find_one(doc! { "email": email }, None).unwrap();
    return email_search == None;
}