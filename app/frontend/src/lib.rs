pub mod components;
pub mod pages;

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