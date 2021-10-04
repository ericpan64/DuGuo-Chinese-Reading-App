// pub mod components;
pub mod pages;
// TODO: merge this into centralized crate for shared "stdlib" values

pub const BASE_URL: &'static str = "http://localhost:8000";

pub enum Phonetic {
    Zhuyin,
    Pinyin,
}

impl Phonetic {
    pub fn as_str(&self) -> &str {
        return match *self {
            Phonetic::Pinyin => "Pinyin",
            Phonetic::Zhuyin => "Zhuyin"
        }
    }
}

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