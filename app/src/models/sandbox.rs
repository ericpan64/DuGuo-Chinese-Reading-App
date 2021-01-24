/*
/// Data Structures not associated with a User account
/// 
/// sandbox.rs
/// ├── SandboxDoc: Strict
/// └── UserFeedback: Struct
*/

use chrono::Utc;
use crate::{
    DatabaseItem,
    config::{SANDBOX_COLL_NAME, USER_FEEDBACK_COLL_NAME},
    html as html_rendering,
    models::zh::{CnType, CnPhonetics}
};
use mongodb::{
    bson::{doc, Bson},
    sync::Database
};
use reqwest;
use scraper;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct SandboxDoc {
    doc_id: String,
    body: String,
    body_html: String,
    // If none, String::new()
    from_url: String,
    cn_type: CnType,
    cn_phonetics: CnPhonetics,
    created_on: String
}

impl DatabaseItem for SandboxDoc {
    fn collection_name(&self) -> &str { return SANDBOX_COLL_NAME; }
    fn primary_key(&self) -> &str { return &self.doc_id; }
}

impl SandboxDoc {
    pub async fn new(body: String, cn_type: String, cn_phonetics: String, url: Option<String>) -> Self {
        let doc_id = Uuid::new_v4().to_string();
        let cn_type = CnType::from_str(&cn_type);
        let cn_phonetics = CnPhonetics::from_str(&cn_phonetics);
        let body_html = html_rendering::convert_string_to_tokenized_html(&body, &cn_type, &cn_phonetics).await;
        let from_url = match url {
            Some(url) => url,
            None => String::new()
        };
        let created_on = Utc::now().to_string();
        let new_doc = SandboxDoc { doc_id, body, body_html, from_url, cn_type, cn_phonetics, created_on };
        return new_doc;
    }

    pub async fn from_url(url: String, cn_type: String, cn_phonetics: String) -> Self {
        // make request
        let resp = reqwest::get(&url).await.unwrap()
            .text().await.unwrap();
        let html = scraper::Html::parse_document(&resp);
        // get body from all headers, paragraphs in-order
        let body_selector = scraper::Selector::parse("body h1,h2,h3,h4,h5,h6,p").unwrap();
        let mut body_text = String::with_capacity(resp.len());
        for item in html.select(&body_selector) {
            body_text += &item.text().collect::<String>();
        }
        return SandboxDoc::new(body_text, cn_type, cn_phonetics, Some(url)).await;
    }

    pub fn get_doc_html_and_phonetics_from_id(db: &Database, doc_id: String) -> Option<(String, String)> {
        let coll = (*db).collection(SANDBOX_COLL_NAME);
        let query_doc = doc! { "doc_id": doc_id };
        let mut doc_html = String::new();
        let mut cn_phonetics = String::new();
        let res = match coll.find_one(query_doc, None).unwrap() {
            Some(doc) => {
                doc_html += doc.get("body_html").and_then(Bson::as_str).expect("No body_html was stored");
                cn_phonetics += doc.get("cn_phonetics").and_then(Bson::as_str).expect("No phonetic info was stored");
                Some((doc_html, cn_phonetics))       
            },
            None => None
        };
        return res;
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserFeedback {
    feedback: String,
    /// If none, String::new()
    contact: String, 
    created_on: String
}

impl DatabaseItem for UserFeedback {
    fn collection_name(&self) -> &str { return USER_FEEDBACK_COLL_NAME; }
    fn primary_key(&self) -> &str { return &self.created_on; }
}

impl UserFeedback {
    pub fn new(feedback: String, contact: String) -> Self {
        let created_on = Utc::now().to_string();
        let new_feedback = UserFeedback { feedback, contact, created_on };
        return new_feedback;
    }
}