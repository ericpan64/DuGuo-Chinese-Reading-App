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
            <span class={self.uid.clone()} tabindex="0" data-bs-toggle="popover" data-bs-content={self.defn_html.clone()} title={self.title_html.clone()}>
                <table name={self.uid.clone()}>
                    <tr>
                        { for self.phonetic_list.iter().enumerate().map(|(i, p)| Self::generate_phonetic_td(p.clone(), self.char_list[i].clone())) }
                    </tr>
                    <tr>
                        { for self.char_list.iter().map(|c| Self::generate_char_td(c.clone())) }
                    </tr>
                </table>
            </span>
        }
    }
}

impl PhraseSpan {
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

    fn generate_phonetic_td(p: String, c: String) -> Html {
        html! { <td class="phonetic" name=c>{p}</td> }
    }

    fn generate_char_td(c: String) -> Html {
        html! { <td class="char">{c}</td> }
    }
}