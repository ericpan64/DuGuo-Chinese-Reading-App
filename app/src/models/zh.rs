/*
/// Data Structures relating to Chinese text
/// 
/// chinese.rs
/// ├── CnType: Enum
/// ├── CnPhonetics: Enum
/// └── CnEnDictEntry: Struct
*/

use crate::CacheItem;
use serde::{Serialize, Deserialize};
use std::{
    collections::HashMap,
    fmt
};
use redis::{
    AsyncCommands,
    aio::Connection
};

/* Enums */
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum CnType {
    Traditional,
    Simplified
}

impl CnType {
    pub fn as_str(&self) -> &str {
        return match *self {
            CnType::Traditional => "Traditional",
            CnType::Simplified => "Simplified"
        };
    }

    pub fn from_str(s: &str) -> Self {
        return match s {
            "Traditional" => CnType::Traditional,
            "traditional" => CnType::Traditional,
            "trad" => CnType::Traditional,
            "Simplified" => CnType::Simplified,
            "simplified" => CnType::Simplified,
            "simp" => CnType::Simplified,
            _ => CnType::Simplified // Default to simplified
        }
    }
}

/// Implements to_string()
impl fmt::Display for CnType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return write!(f, "{}", self.as_str());
    }
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum CnPhonetics {
    Pinyin,
    Zhuyin
}

impl CnPhonetics {
    pub fn as_str(&self) -> &str {
        return match *self {
            CnPhonetics::Pinyin => "Pinyin",
            CnPhonetics::Zhuyin => "Zhuyin"
        };
    }

    pub fn from_str(s: &str) -> Self {
        return match s {
            "Pinyin" => CnPhonetics::Pinyin,
            "pinyin" => CnPhonetics::Pinyin,
            "Zhuyin" => CnPhonetics::Zhuyin,
            "zhuyin" => CnPhonetics::Zhuyin,
            "Bopomofo" => CnPhonetics::Zhuyin,
            "bopomofo" => CnPhonetics::Zhuyin,
            _ => CnPhonetics::Pinyin // Default to pinyin
        }
    }
}

/// Implements to_string()
impl fmt::Display for CnPhonetics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return write!(f, "{}", self.as_str());
    }
}

/* Structs */
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CnEnDictEntry {
    uid: String,
    trad: String,
    simp: String,
    raw_pinyin: String,
    formatted_pinyin: String,
    pub trad_html: String,
    pub simp_html: String,
    def: String,
    zhuyin: String,
    pub trad_zhuyin_html: String,
    pub simp_zhuyin_html: String,
    pub radical_map: String
}

/// For CnEnDictEntry, the current uid is generated using: vec![simp, raw_pinyin]
impl CacheItem for CnEnDictEntry { }

impl CnEnDictEntry {
    pub async fn from_uid(conn: &mut Connection, uid: String) -> Self {
        let query_map = (*conn).hgetall::<&str, HashMap<String, String>>(&uid).await.unwrap();
        let res = match query_map.len() {
            0 => CnEnDictEntry::generate_lookup_failed_entry(&uid),
            _ => CnEnDictEntry {
                    uid,
                    trad : query_map.get("trad").unwrap().to_owned(),
                    simp: query_map.get("simp").unwrap().to_owned(),
                    raw_pinyin: query_map.get("raw_pinyin").unwrap().to_owned(),
                    formatted_pinyin: query_map.get("formatted_pinyin").unwrap().to_owned(),
                    trad_html: query_map.get("trad_html").unwrap().to_owned(),
                    simp_html: query_map.get("simp_html").unwrap().to_owned(),
                    def: query_map.get("def").unwrap().to_owned(),
                    zhuyin: query_map.get("zhuyin").unwrap().to_owned(),
                    trad_zhuyin_html: query_map.get("trad_zhuyin_html").unwrap().to_owned(),
                    simp_zhuyin_html: query_map.get("simp_zhuyin_html").unwrap().to_owned(),
                    radical_map: query_map.get("radical_map").unwrap().to_owned(),
                }
        };
        return res;
    }

    pub async fn from_tokenizer_components(conn: &mut Connection, simp: &str, raw_pinyin: &str) -> Self {
        let uid = CnEnDictEntry::generate_uid(vec![simp.to_string(), raw_pinyin.to_string()]);
        return CnEnDictEntry::from_uid(conn, uid).await;
    }

    pub fn lookup_failed(&self) -> bool {
        return self.trad_html == "";
    }

    pub fn get_vocab_data(&self, cn_type: &CnType, cn_phonetics: &CnPhonetics) -> (String, String, String, String) {
        // Order: (phrase, defn, phrase_phonetics, phrase_html)
        let defn = &self.def;
        let (phrase, phrase_phonetics, phrase_html) = match (cn_type, cn_phonetics) {
            (CnType::Traditional, CnPhonetics::Pinyin) => (&self.trad, &self.formatted_pinyin, &self.trad_html),
            (CnType::Traditional, CnPhonetics::Zhuyin) => (&self.trad, &self.zhuyin, &self.trad_zhuyin_html),
            (CnType::Simplified, CnPhonetics::Pinyin) => (&self.simp, &self.formatted_pinyin, &self.simp_html),
            (CnType::Simplified, CnPhonetics::Zhuyin) => (&self.simp, &self.zhuyin, &self.simp_zhuyin_html)
        };
        return (phrase.to_string(), defn.to_string(), phrase_phonetics.to_string(), phrase_html.to_string());
    }

    fn generate_lookup_failed_entry(uid: &str) -> Self {
        const LOOKUP_ERROR_MSG: &str = "N/A - Not found in database";
        let res = CnEnDictEntry {
            uid: String::from(uid),
            def: String::from(LOOKUP_ERROR_MSG),
            ..Default::default()
        }; 
        return res;
    }
}