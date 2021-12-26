/*
/// This file is a "dummy" config file provided for reference and if you want to deploy the app locally.
/// All confidential information is absent from this document.
*/

/* Static Vars */
pub static DB_URI: &str = "mongodb://root:example@mongodb:27017/";
pub static DB_NAME: &str = "duguo";
pub static REDIS_URI: &str = "redis://redis-cache:6379/";
pub static USER_COLL_NAME: &str = "users";
pub static SANDBOX_COLL_NAME: &str = "sandbox";
pub static USER_DOC_COLL_NAME: &str = "docs";
pub static USER_VOCAB_COLL_NAME: &str = "vocab";
pub static USER_VOCAB_LIST_COLL_NAME: &str = "vocab-list";
pub static USER_FEEDBACK_COLL_NAME: &str = "feedback";
pub static TOKENIZER_PORT: u16 = 8881;
pub static TOKENIZER_HOSTNAME: &str = "duguo-tokenizer"; // Container name from docker-compose.yml
pub static JWT_NAME: &str = "duguo-代币";
pub static JWT_SECRET: &[u8; 20] = b"somesupersecretthing";
pub static JWT_LIFETIME: i64 = 24 * 7; // 1 week (match with Rocket cookie length)