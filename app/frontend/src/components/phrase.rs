use yew::prelude::*;
use crate::{CnType, CnPhonetics, CnEnDictEntry, CnPhrase};

#[derive(Properties, Clone, PartialEq)]
pub struct SpanProps {
    pub phrase: CnPhrase,
    pub has_learned: bool,
    pub cn_type: CnType,
    pub cn_phonetics: CnPhonetics
}

pub struct PhraseSpan {
    uid: String,
    defn_html: String,
    title_html: String,
    phonetic_list: Vec<String>,
    char_list: Vec<String>,
    props: SpanProps
}

// TODO: Implement this
pub enum Msg {
    SavePhrase,
    SpeakPhrase
}

impl Component for PhraseSpan {
    type Message = ();
    type Properties = SpanProps;

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self { 
        const VOICE_ICON: &str = "<img src=&quot;/static/img/volume-up-fill.svg&quot;></img>";
        const DOWNLOAD_ICON: &str = "<img src=&quot;/static/img/download.svg&quot;></img>";
        if !props.phrase.lookup_success {
            return Self::generate_not_found_phrase(props);
        } 
        let uid = props.phrase.entry.uid.clone();
        let chars = match props.cn_type {
            CnType::Simplified => &props.phrase.entry.simp,
            CnType::Traditional => &props.phrase.entry.trad
        };
        let (defn_phonetics, view_phonetics) = match props.cn_phonetics {
            CnPhonetics::Pinyin => (&props.phrase.entry.raw_pinyin, &props.phrase.entry.formatted_pinyin),
            CnPhonetics::Zhuyin => (&props.phrase.entry.zhuyin, &props.phrase.entry.zhuyin)
        };
        let defn_html = Self::format_defn_html(&props.phrase.entry);
        let title_html = format!("{} [{}] <a role=&quot;button&quot; href=&quot;#~{}&quot;>{}</a> <a role=&quot;button&quot; href=&quot;#{}&quot;>{}</a>",
            chars, defn_phonetics, props.phrase.entry.simp, VOICE_ICON, uid, DOWNLOAD_ICON
        );
        let phonetic_list: Vec<String> = view_phonetics.split(" ").map(|s| String::from(s)).collect();
        let char_list: Vec<String> = chars.chars().map(|c| c.to_string()).collect();
        Self { uid, defn_html, title_html, phonetic_list, char_list, props  } 
    }
    fn update(&mut self, _msg: Self::Message) -> ShouldRender { false }
    fn change(&mut self, props: Self::Properties) -> ShouldRender { 
        if self.props != props {
            self.props = props;
            true
        } else {
            false
        }
    }
    fn view(&self) -> Html {
        html! {
            <span class="m-1" tabindex="0" data-bs-toggle="popover" data-bs-content={self.defn_html.clone()} title={self.title_html.clone()}>
                <ruby class="m-2" name={self.uid.clone()}>
                    { for self.char_list.iter().enumerate().map(|(i, p)| Self::generate_char_ruby(p.clone(), self.phonetic_list.get(i))) }
                </ruby>
            </span>
        }
    }
}

impl PhraseSpan {
    fn generate_not_found_phrase(props: SpanProps) -> Self {
        let uid = props.phrase.entry.uid.clone();
        let chars = props.phrase.raw_phrase.clone();
        let defn_html = String::from("Not found in CEDICT");
        let title_html = format!("{} [{}] - Not found in CEDICT", chars, &props.phrase.raw_phonetics);
        let phonetic_list: Vec<String> = Vec::new();
        let char_list: Vec<String> = chars.chars().map(|c| c.to_string()).collect();
        Self { uid, defn_html, title_html, phonetic_list, char_list, props  } 
    }
    
    fn format_defn_html(entry: &CnEnDictEntry) -> String {
        const DEFN_DELIM: char = '/'; // Used to separate the description for a single concept definition
        const MULTI_DEFN_DELIM: char = '$'; // Used when a single phrase represents multiple different concepts
        let mut res = String::with_capacity(2500);
        let all_defns: Vec<&str> = entry.defn.split(MULTI_DEFN_DELIM).collect();
        for (i, defns) in all_defns.iter().enumerate() {
            let defns = &defns[1..defns.len()-1];
            let defns = defns.replace("\"", "\'");
            let defn_vec: Vec<&str> = defns.split(DEFN_DELIM).collect();
            for (j, li) in defn_vec.iter().enumerate() {
                res += format!("{}. {}", j+1, li).as_str();
                if j != defn_vec.len() - 1 {
                    res += "<br>";
                } else if i != all_defns.len() - 1 {
                    res += "<hr>"
                }
            }
        }
        return res;
    }

    fn generate_char_ruby(c: String, p: Option<&String>) -> Html {
        let phonetic = match p {
            Some(s) => String::from(s),
            None => String::new()
        };
        html! { <>{c}<rp>{"("}</rp><rt>{phonetic}</rt><rp>{")"}</rp></> }
    }
}