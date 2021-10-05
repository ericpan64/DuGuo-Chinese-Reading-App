use yew_router::prelude::*;

pub mod components;
pub mod pages;

// TODO: merge Enums into centralized crate for shared "stdlib" values
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

#[derive(Switch, Clone, Debug)]
pub enum Route {
    #[to="/login"]
    Login,
    #[to="/feedback"]
    Feedback,
    #[to="/sandbox"]
    Sandbox,
    #[to="/reader"]
    Reader,
    #[to="/u/{uid}"]
    Profile(String),
    #[to="/"]
    Home,
}
