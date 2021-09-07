use yew::prelude::*;
use super::super::{CnType, Phonetic};

pub struct Sandbox {
    link: ComponentLink<Self>,
    phonetic_type: Phonetic,
    cn_type: CnType,
}


pub enum Msg {
    UpdatePhonetic(Phonetic),
    UpdateCnType(CnType),
    SwitchToLoadingButton,
    SubmitForm,
}

impl Component for Sandbox {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let phonetic_type = Phonetic::Pinyin;
        let cn_type = CnType::Simplified;
        Self { link, phonetic_type, cn_type }
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::UpdatePhonetic(new_type) => {
                self.phonetic_type = new_type;
                true
            },
            Msg::UpdateCnType(new_type) => {
                self.cn_type = new_type;
                true
            },
            Msg::SwitchToLoadingButton => {
                false
            },
            Msg::SubmitForm => {
                false
            }
        }
    }
    fn change(&mut self, _: Self::Properties) -> ShouldRender { false }
    fn view(&self) -> Html {
        html! {
            <>
                {self.view_header()}
            </>
        }
    }
}

impl Sandbox {
    fn view_header(&self) -> Html {
        let render_phonetic_text = match self.phonetic_type {
            Phonetic::Pinyin => "Render Pinyin",
            Phonetic::Zhuyin => "Render Zhuyin",
        };
        let render_cn_text = match self.cn_type {
            CnType::Simplified => "Render Simplified",
            CnType::Traditional => "Render Traditional",
        };
        html! {
            <header class="page-header page-header-light bg-white">
                <div class="page-header-content">
                    <div class="container">
                        <h1>{"Sandbox"}</h1>
                        <p>{"Try-out the app by uploading some text! Adjust settings based on how you would like the output to render."}</p>
                        <p>{"Any Chinese text should render - by default the server attempts to lookup Simplified, and then Traditional."}</p>
                        <span>
                            <button class="btn btn-primary dropdown-toggle" id="phonetic-setting" type="button" data-bs-toggle="dropdown" aria-expanded="false">
                                {render_phonetic_text}
                            </button>
                            <ul class="dropdown-menu" aria-labelledby="dropdownMenuButton">
                                <li><a class="dropdown-item" onclick=self.link.callback(|_| Msg::UpdatePhonetic(Phonetic::Pinyin))>{"Render Pinyin"}</a></li>
                                <li><a class="dropdown-item" onclick=self.link.callback(|_| Msg::UpdatePhonetic(Phonetic::Zhuyin))>{"Render Zhuyin (Bopomofo)"}</a></li>
                            </ul>
                        </span>
                        <span>
                            <button class="btn btn-primary dropdown-toggle" id="char-setting" type="button" data-bs-toggle="dropdown" aria-expanded="false">
                                {render_cn_text}
                            </button>
                            <ul class="dropdown-menu" aria-labelledby="dropdownMenuButton">
                                <li><a class="dropdown-item" onclick=self.link.callback(|_| Msg::UpdateCnType(CnType::Simplified))>{"Render Simplified"}</a></li>
                                <li><a class="dropdown-item" onclick=self.link.callback(|_| Msg::UpdateCnType(CnType::Traditional))>{"Render Traditional"}</a></li>
                            </ul>
                        </span>
                        <br/><br/>
                        <form action="/api/sandbox-upload" id="upload" onsubmit=self.link.callback(|_| Msg::SwitchToLoadingButton) method="POST">
                            <textarea name="text" form="upload" rows="5" cols="35" required=true>{"希望这个网站能帮助您多读中文！"}</textarea>
                            <br/><br/>
                            <button id="upload-button" class="btn btn-outline-primary" type="submit">{"Upload Text"}</button>
                        </form>
                        <br/>
                        <p>{"Or try uploading a URL to a Chinese article (news, leisure, etc.). If you're feeling lucky, "}<a href="https://zh.wikipedia.org/wiki/Special:%E9%9A%8F%E6%9C%BA%E9%A1%B5%E9%9D%A2" target="_blank">{"here's"}</a>{" a link to a random Chinese Wikipedia article."}</p>
                        <form class="form" action="/api/sandbox-url-upload" id="sandbox-url-form" onsubmit=self.link.callback(|_| Msg::SwitchToLoadingButton) method="POST">
                            <input type="text" name="url" placeholder="Article URL" required=true />
                            <br/><br/>
                            <button id="url-upload-button" class="btn btn-outline-primary" type="submit">{"Upload using URL"}</button>
                        </form>
                    </div>
                </div>
            </header>
        }
    }

}
