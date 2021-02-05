/*
/// For Data Structures not associated with a User account.
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
    source: String,
    cn_type: CnType,
    cn_phonetics: CnPhonetics,
    created_on: String
}

impl DatabaseItem for SandboxDoc {
    fn collection_name(&self) -> &str { return SANDBOX_COLL_NAME; }
    fn primary_key(&self) -> &str { return &self.doc_id; }
}

impl SandboxDoc {
    /// Generates a new SandboxDoc. A uuid is generated and assigned.
    pub async fn new(body: String, cn_type: String, cn_phonetics: String, source: String) -> Self {
        let doc_id = Uuid::new_v4().to_string();
        let cn_type = CnType::from_str(&cn_type).unwrap();
        let cn_phonetics = CnPhonetics::from_str(&cn_phonetics).unwrap();
        let body_html = html_rendering::convert_string_to_tokenized_html(&body, &cn_type, &cn_phonetics, None).await;
        let created_on = Utc::now().to_string();
        let new_doc = SandboxDoc { doc_id, body, body_html, source, cn_type, cn_phonetics, created_on };
        return new_doc;
    }

    /// Generates a new SandboxDoc using HTML-parsed text from the specified URL.
    pub async fn from_url(url: String, cn_type: String, cn_phonetics: String) -> Self {
        let resp = reqwest::get(&url).await.unwrap()
            .text().await.unwrap();
        let html = scraper::Html::parse_document(&resp);
        // Grabs body headers+paragraphs in-order
        let body_selector = scraper::Selector::parse("body h1,h2,h3,h4,h5,h6,p").unwrap();
        let mut body_text = String::with_capacity(resp.len());
        for item in html.select(&body_selector) {
            body_text += &item.text().collect::<String>();
        }
        return SandboxDoc::new(body_text, cn_type, cn_phonetics, url).await;
    }

    /// Called in primary.rs to get appropriate SandboxDoc info for viewing.
    /// TODO: rename this, and see if it can be made generic via DatabaseItem defn
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
    /// Creates a UserFeedback object. This is generally anonymous.
    /// TODO: rename this to AppFeedback
    pub fn new(feedback: String, contact: String) -> Self {
        let created_on = Utc::now().to_string();
        let new_feedback = UserFeedback { feedback, contact, created_on };
        return new_feedback;
    }
}