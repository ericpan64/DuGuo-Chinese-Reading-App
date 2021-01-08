/// This file is a "dummy" config file provided for reference and if you want to deploy the app locally.
/// All confidential information is absent from this document.

use jsonwebtoken::{encode, Header, EncodingKey, decode, DecodingKey, Validation, Algorithm};
use chrono::{Duration, Utc, DateTime};
use serde::{Serialize, Deserialize};
use mongodb::{
    bson::{doc,to_document},
    Database,
};

/* Static Vars */
pub static DB_URI: &str = "mongodb://root:example@mongodb:27017/";
pub static DB_NAME: &str = "duguo";
pub static USER_COLL_NAME: &str = "users";
pub static SANDBOX_COLL_NAME: &str = "sandbox";
pub static USER_DOC_COLL_NAME: &str = "docs";
pub static USER_VOCAB_COLL_NAME: &str = "vocab";
pub static USER_VOCAB_LIST_COLL_NAME: &str = "vocab-list";
pub static USER_FEEDBACK_COLL_NAME: &str = "feedback";
pub static CEDICT_COLL_NAME: &str = "cedict";
pub static TOKENIZER_PORT: u16 = 8881;
pub static TOKENIZER_HOSTNAME: &str = "tokenizer-server"; // tokenizer-server=Container name from docker-compose.yml

static SECRET_FOR_JWT: &[u8; 20] = b"somesupersecretthing";
static LIFETIME_IN_HOURS_FOR_JWT: i64 = 24 * 7; // 1 week (match with Rocket cookie length)


/* Structs */
// _ JWT _
#[derive(Serialize, Deserialize, Debug)]
struct UserCredentials {
    username: String,
    pw_hash: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct UserToken {
    cred: UserCredentials,
    iat: i64,
    exp: i64,
}

/* Public Function */
pub fn str_to_hashed_string(str_to_hash: &str) -> String {
    // Actually do hash in PRD
    return str_to_hash.to_string();
}

pub fn generate_jwt(username: String, password: String) -> Result<String, jsonwebtoken::errors::Error> {
    let jwt_header: Header = Header::default();
    let jwt_encoding_key: EncodingKey = EncodingKey::from_secret(SECRET_FOR_JWT);
    
    let pw_hash = str_to_hashed_string(password.as_str());
    let cred = UserCredentials {
        username,
        pw_hash,
    };
    let current_datetime: DateTime<Utc> = Utc::now();
    let expire_datetime: DateTime<Utc> = Utc::now() + Duration::hours(LIFETIME_IN_HOURS_FOR_JWT);
    let token = UserToken {
        cred: cred,
        iat: current_datetime.timestamp(),
        exp: expire_datetime.timestamp(),
    };
    return encode(&jwt_header, &token, &jwt_encoding_key);
}

pub async fn validate_jwt_and_get_username(db: &Database, token: &str) -> Option<String> {
    let jwt_decoding_key: DecodingKey = DecodingKey::from_secret(SECRET_FOR_JWT);
    let jwt_validation_algorithm: Validation = Validation::new(Algorithm::HS256); // matches jwt_header

    let res = match decode::<UserToken>(token, &jwt_decoding_key, &jwt_validation_algorithm) {
        Ok(data) => {
            let payload: UserToken = data.claims;
            let cred = payload.cred;
            let exp = payload.exp;
            let is_active = check_if_jwt_is_active(exp);
            let username = match is_active {
                true => get_username_from_valid_user_credentials(db, cred).await,
                false => None
            };
            username
        }
        Err(e) => {
            println!("Error when validating JWT: {:?}", e);
            None
        }
    };
    return res;
}

/* Private Functions */
fn check_if_jwt_is_active(expiration_timestamp: i64) -> bool {
    let current_timestamp: i64 = Utc::now().timestamp();
    return current_timestamp <= expiration_timestamp;
}

async fn get_username_from_valid_user_credentials(db: &Database, cred: UserCredentials) -> Option<String> {
    let coll = (*db).collection(USER_COLL_NAME);
    let cred_as_document = to_document(&cred).unwrap();
    let user_search = coll.find_one(cred_as_document, None).await.unwrap();
    let res = match user_search {
        Some(_) => Some(cred.username),
        None => None
    };
    return res;
}
