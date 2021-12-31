/*
/// Data Structures not associated with a User account.
*/

use chrono::Utc;
use crate::{
    DatabaseItem,
    scrape_text_from_url,
    html_rendering,
    convert_string_to_tokenized_phrases,
    config::{SANDBOX_COLL_NAME, USER_FEEDBACK_COLL_NAME},
    models::zh::{CnType, CnPhonetics, CnPhrase}
};
use mongodb::bson::doc;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct SandboxDoc {
    doc_id: String,
    body: String,
    pub body_html: String,
    tokenized_body_json: Vec<CnPhrase>,
    source: String,
    cn_type: CnType,
    pub cn_phonetics: CnPhonetics,
    created_on: String
}

impl DatabaseItem for SandboxDoc {
    fn collection_name() -> &'static str { return SANDBOX_COLL_NAME; }
    fn all_field_names() -> Vec<&'static str> { 
        return vec!["doc_id", "body", "tokenized_body_json", "source", 
            "cn_type", "cn_phonetics", "created_on"]; 
    }
    fn primary_key(&self) -> &str { return &self.doc_id; }
}

impl SandboxDoc {
    /// Generates a new SandboxDoc. A uuid is generated and assigned.
    pub async fn new(body: String, cn_type: String, cn_phonetics: String, source: String) -> Self {
        let doc_id = Uuid::new_v4().to_string();
        let cn_type = CnType::from_str(&cn_type).unwrap();
        let cn_phonetics = CnPhonetics::from_str(&cn_phonetics).unwrap();
        let created_on = Utc::now().to_string();
        let body_html = html_rendering::convert_string_to_tokenized_html(&body, &cn_type, &cn_phonetics).await;
        let tokenized_body_json = convert_string_to_tokenized_phrases(&body).await;
        let new_doc = SandboxDoc { doc_id, body, body_html, tokenized_body_json, source, cn_type, cn_phonetics, created_on };
        return new_doc;
    }

    /// Generates a new SandboxDoc using HTML-parsed text from the specified URL.
    pub async fn from_url(url: String, cn_type: String, cn_phonetics: String) -> Self {
        let (_, body_text) = scrape_text_from_url(&url).await;
        return SandboxDoc::new(body_text, cn_type, cn_phonetics, url).await;
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppFeedback {
    feedback: String,
    contact: String, 
    created_on: String
}

impl DatabaseItem for AppFeedback {
    fn collection_name() -> &'static str { return USER_FEEDBACK_COLL_NAME; }
    fn all_field_names() -> Vec<&'static str> { 
        return vec!["feedback", "contact", "created_on"];
    }
    fn primary_key(&self) -> &str { return &self.created_on; }
}

impl AppFeedback {
    /// Creates an AppFeedback object. This is generally anonymous.
    pub fn new(feedback: String, contact: String) -> Self {
        let created_on = Utc::now().to_string();
        let new_feedback = AppFeedback { feedback, contact, created_on };
        return new_feedback;
    }
}