use yew_router::prelude::*;

pub mod components;
pub mod pages;

// TODO: merge Enums into centralized crate for shared "stdlib" values
#[derive(Clone, Default, PartialEq)]
pub struct CnPhrase {
    pub entry: CnEnDictEntry,
    pub lookup_success: bool,
    pub raw_phrase: String,
    pub raw_phonetics: String,
}

#[derive(Clone, Default, PartialEq)]
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

#[derive(Clone, PartialEq)]
pub enum CnPhonetics {
    Zhuyin,
    Pinyin,
}

impl CnPhonetics {
    pub fn as_str(&self) -> &str {
        return match *self {
            CnPhonetics::Pinyin => "Pinyin",
            CnPhonetics::Zhuyin => "Zhuyin"
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum CnType {
    Traditional,
    Simplified,
}

impl CnType {
    pub fn as_str(&self) -> &str {
        return match *self {
            CnType::Traditional => "Traditional",
            CnType::Simplified => "Simplified"
        };
    }
}

#[derive(Switch, Clone, Debug)]
pub enum Route {
    #[to="/login"]
    Login,
    #[to="/feedback"]
    Feedback,
    #[to="/sandbox"]
    Sandbox,
    #[to="/reader/{uid}"]
    Reader(String),
    #[to="/u/{uid}"]
    Profile(String),
    #[to="/"]
    Home,
}
