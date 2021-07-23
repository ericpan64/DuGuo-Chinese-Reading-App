/*
/// Data Structures relating to Chinese text.
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
    pub fn from_str(s: &str) -> Option<Self> {
        return match s.to_ascii_lowercase().as_str() {
            "traditional" => Some(CnType::Traditional),
            "trad" => Some(CnType::Traditional),
            "simplified" => Some(CnType::Simplified),
            "simp" => Some(CnType::Simplified),
            _ => None
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
    pub fn from_str(s: &str) -> Option<Self> {
        return match s.to_ascii_lowercase().as_str() {
            "pinyin" => Some(CnPhonetics::Pinyin),
            "zhuyin" => Some(CnPhonetics::Zhuyin),
            "bopomofo" => Some(CnPhonetics::Zhuyin),
            _ => None
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
    pub uid: String,
    pub trad: String,
    pub simp: String,
    pub raw_pinyin: String,
    pub formatted_pinyin: String,
    pub defn: String,
    pub zhuyin: String,
    pub radical_map: String
}

/// For CnEnDictEntry, the current uid is generated using: vec![simp, raw_pinyin]
impl CacheItem for CnEnDictEntry {
    fn get_uid_ordered_values(&self) -> Vec<&str> { return vec![&self.simp, &self.raw_pinyin]; }
}

impl CnEnDictEntry {
    /// Looks up a CEDICT entry in Redis using the specified uid.
    /// Defaults to generate_lookup_failed_entry().
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
                    defn: query_map.get("defn").unwrap().to_owned(),
                    zhuyin: query_map.get("zhuyin").unwrap().to_owned(),
                    radical_map: query_map.get("radical_map").unwrap().to_owned(),
                }
        };
        return res;
    }
    /// Returns true if object is a "failed lookup" entry, false otherwise.
    pub fn lookup_failed(&self) -> bool {
        return self.formatted_pinyin == "";
    }
    /// Generates generic "failed lookup" entry.
    /// The uid is preserved so the failed case can be identified.
    /// The LOOKUP_ERROR_STR is used for compatibility with /api/delete-vocab/NA.
    fn generate_lookup_failed_entry(uid: &str) -> Self {
        const LOOKUP_ERROR_MSG: &str = "NA - Not found in database";
        const LOOKUP_ERROR_STR: &str = "NA";
        let res = CnEnDictEntry {
            uid: String::from(uid),
            trad: String::from(LOOKUP_ERROR_STR),
            simp: String::from(LOOKUP_ERROR_STR),
            radical_map: String::from(LOOKUP_ERROR_STR),
            defn: String::from(LOOKUP_ERROR_MSG),
            ..Default::default()
        }; 
        return res;
    }
}