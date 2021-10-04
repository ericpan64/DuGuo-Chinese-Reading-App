/*
/// Module handling user authentication and cookies
/// 
/// TODO: Update these docs across files once implementation is complete
///
/// auth.rs
/// ├── UserCredentials: Struct
/// ├── UserToken: Struct
/// |
/// └── pub fn:
///     └── generate_http_cookie
///     └── add_user_cookie_to_context
///     └── get_username_from_cookie
///     └── str_to_hashed_string
*/

use blake2::{Blake2b, Digest};
use crate::{
    DatabaseItem,
    config::{JWT_LIFETIME, JWT_NAME, JWT_SECRET, USER_COLL_NAME},
    models::user::User
};
use chrono::{Duration, Utc, DateTime};
use jsonwebtoken::{Header, EncodingKey, DecodingKey, Validation, Algorithm};
use mongodb::{
    bson::{self, doc},
    sync::Database
};
use rocket::http::{Cookie, Cookies, SameSite};
use serde::{Serialize, Deserialize};
use std::{
    collections::HashMap,
    error::Error
};

/* Structs */
#[derive(Serialize, Deserialize, Debug)]
struct UserCredentials {
    username: String,
    pw_hash: String
}

#[derive(Serialize, Deserialize, Debug)]
struct UserToken {
    cred: UserCredentials,
    iat: i64,
    exp: i64
}

/* Public Functions */
/// Generates browser cookie (storing a JWT) with appropriate settings.
pub fn generate_http_cookie(db: &Database, username: String, password: String) -> Cookie<'static> {
    let jwt = match generate_jwt(db, username, password) {
        Ok(token) => token,
        Err(e) => {
            eprintln!("Error when generating jwt: {:?}", e);
            String::new()
        }
    };
    let mut cookie = Cookie::new(JWT_NAME, jwt);
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Strict);
    cookie.set_path("/");
    return cookie;
}

/// Looks for a DuGuo JWT in the user browser cookies, then adds validated username to context.
/// Returns true if a username was added, false otherwise.
pub fn add_user_cookie_to_context(cookies: &Cookies<'_>, db: &Database, context: &mut HashMap<&str, String>) -> bool {
    let cookie_lookup = (*cookies).get(JWT_NAME);
    let username_from_cookie = get_username_from_cookie(db, cookie_lookup);
    let res = match username_from_cookie {
        Some(username) => { 
            (*context).insert("username", username);
            true
        },
        None => false
    };
    return res;
}

/// Returns a validated DuGuo username from the cookie.
pub fn get_username_from_cookie(db: &Database, cookie_lookup: Option<&Cookie<'static>>) -> Option<String> {
    let mut res = None;
    if let Some(ref login_cookie) = cookie_lookup {
        let jwt_to_check = login_cookie.value();
        res = validate_jwt_and_get_username(db, jwt_to_check);
    }
    return res;
}

/// Hashes input string. Used for password storage. Salt is randomly generated (defined in models/user.rs).
pub fn str_to_hashed_string(str_to_hash: &str, salt: &str) -> String {
    let mut hasher = Blake2b::new();
    hasher.update(str_to_hash.as_bytes());
    hasher.update(b"$");
    hasher.update(salt);
    let res = hasher.finalize().to_vec();
    return hex::encode(res);
}

/* Private Functions */
/// Generates JWT with appropriate configuration.
fn generate_jwt(db: &Database, username: String, password: String) -> Result<String, Box<dyn Error>> {
    let jwt_header: Header = Header::default();
    let jwt_encoding_key: EncodingKey = EncodingKey::from_secret(JWT_SECRET);
    let pw_salt = User::get_values_from_query(&db, 
        doc!{ "username": &username }, 
        vec!["pw_salt"])[0].to_owned();
    let pw_hash = str_to_hashed_string(&password, &pw_salt);
    let cred = UserCredentials {
        username,
        pw_hash,
    };
    let current_datetime: DateTime<Utc> = Utc::now();
    let expire_datetime: DateTime<Utc> = Utc::now() + Duration::hours(JWT_LIFETIME);
    let token = UserToken {
        cred: cred,
        iat: current_datetime.timestamp(),
        exp: expire_datetime.timestamp(),
    };
    let jwt = jsonwebtoken::encode(&jwt_header, &token, &jwt_encoding_key)?;
    return Ok(jwt);
}

/// Performs the JWT validation and username retrieval steps.
fn validate_jwt_and_get_username(db: &Database, token: &str) -> Option<String> {
    let jwt_decoding_key: DecodingKey = DecodingKey::from_secret(JWT_SECRET);
    let jwt_validation_algorithm: Validation = Validation::new(Algorithm::HS256); // matches jwt_header

    let res = match jsonwebtoken::decode::<UserToken>(token, &jwt_decoding_key, &jwt_validation_algorithm) {
        Ok(data) => {
            let payload: UserToken = data.claims;
            let cred = payload.cred;
            let exp = payload.exp;
            let is_active = check_if_jwt_is_active(exp);
            let username = match is_active {
                true => get_username_from_valid_user_credentials(db, cred),
                false => None
            };
            username
        }
        Err(e) => {
            eprintln!("Error when validating JWT: {:?}", e);
            None
        }
    };
    return res;
}

/// Verifies that JWT has not expired (true if not expired, false if expired).
fn check_if_jwt_is_active(expiration_timestamp: i64) -> bool {
    let current_timestamp: i64 = Utc::now().timestamp();
    return current_timestamp <= expiration_timestamp;
}

/// Returns username in given UserCredentials if it is successfully found in the MongoDB.
fn get_username_from_valid_user_credentials(db: &Database, cred: UserCredentials) -> Option<String> {
    let coll = (*db).collection(USER_COLL_NAME);
    let cred_as_document = bson::to_document(&cred).unwrap();
    let user_search = coll.find_one(cred_as_document, None).unwrap();
    let res = match user_search {
        Some(_) => Some(cred.username),
        None => None
    };
    return res;
}