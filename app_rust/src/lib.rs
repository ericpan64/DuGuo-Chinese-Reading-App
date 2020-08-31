enum CnType {
    Traditional,
    Simplified,
}

/// cnPhrase represents a Chinese phrase composing of 1 or more characters.
/// Example: `你好`!
struct CnPhrase {
    phrase: String,
    pinyin: String,
    def: String,
    lang_type: CnType,
}

struct CnDictEntry {
    trad: CnPhrase,
    simp: CnPhrase,
    pinyin: String,
    def: String,
}

struct User {
    id: u32,
    pw_hash: String,
}

struct Document {
    body: String,
    title: String,
    context: String,
}

#[cfg(test)]
mod tests {

    #[test]
    #[should_panic]
    fn it_works() {
        assert_eq!("let's", "get it");
    }

}