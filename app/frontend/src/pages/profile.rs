use yew::prelude::*;
use super::super::{CnType, Phonetic};

pub struct Profile {
    link: ComponentLink<Self>,
    phonetic_type: Phonetic,
    cn_type: CnType,
    uid: String,
}

pub enum Msg {
    UpdatePhonetic(Phonetic), 
    UpdateCnType(CnType),
    SwitchToLoadingButton,
    DownloadDocs,
    DownloadVocab,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub uid: String,
}

impl Component for Profile {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let phonetic_type = Phonetic::Pinyin;
        let cn_type = CnType::Simplified;
        let uid = props.uid;
        Self { link, phonetic_type, cn_type, uid }
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            _ => false
        }
    }
    fn change(&mut self, _: Self::Properties) -> ShouldRender { false }
    fn view(&self) -> Html {
        html! {
            <header class="page-header page-header-light bg-white">
            <div class="page-header-content">
                <div class="container">
                    <h1>{self.uid.as_str()}</h1>
                        <hr/>
                        <h5>{"User Settings"}</h5>
                        <ul class="nav nav-pills">
                            <li class="nav-item dropdown">
                                <a class="nav-link dropdown-toggle" data-bs-toggle="dropdown" role="button" aria-expanded="false">{format!("Phonetic: {}", self.phonetic_type.as_str())}</a>
                                <ul class="dropdown-menu">
                                    <li><a class="dropdown-item" onclick=self.link.callback(|_| Msg::UpdatePhonetic(Phonetic::Pinyin))>{"View Pinyin"}</a></li>
                                    <li><a class="dropdown-item" onclick=self.link.callback(|_| Msg::UpdatePhonetic(Phonetic::Zhuyin))>{"View Zhuyin (Bopomofo)"}</a></li>    
                                </ul>
                            </li>
                            <li class="nav-item dropdown">
                                <a class="ml-2 nav-link dropdown-toggle" data-bs-toggle="dropdown" role="button" aria-expanded="false">{format!("Charset: {}", self.cn_type.as_str())}</a>
                                <ul class="dropdown-menu">
                                    <li><a class="dropdown-item" onclick=self.link.callback(|_| Msg::UpdateCnType(CnType::Simplified))>{"View Simplified"}</a></li>
                                    <li><a class="dropdown-item" onclick=self.link.callback(|_| Msg::UpdateCnType(CnType::Traditional))>{"View Traditional"}</a></li>
                                </ul>
                            </li>
                        </ul>

                        <h4 class="mt-4">{"Upload Document"}</h4>
                        <hr/>
                        <h5>{"Parse from URL"}</h5>
                        <form class="form" action="/api/upload" id="user-url-form" onsubmit=self.link.callback(|_| Msg::SwitchToLoadingButton) method="POST">
                            <input type="text" name="url" placeholder="Document URL" required=true/>
                            <button id="url-upload-button" class="ml-2 btn btn-outline-primary" type="submit">{"Upload URL"}</button>
                        </form>
                        <h5>{"Copy & Paste"}</h5>
                        <form class="form" action="/api/upload" id="user-doc-form" onsubmit=self.link.callback(|_| Msg::SwitchToLoadingButton) method="POST">
                            <input class="mt-2" type="text" name="title" placeholder="Document Title" required=true/><br/>
                            <input class="mt-2" type="text" name="source" placeholder="Document Source (optional)"/>
                            <textarea class="mt-2 form-control" name="body" form="user-doc-form" rows="2" cols="15" required=true></textarea>
                            <button id="doc-upload-button" class="mt-2 btn btn-outline-primary" type="submit">{"Upload Text"}</button>
                        </form>
                        <h4 class="mt-4">{"View Documents + Vocab"}</h4>
                        <hr/>
                        <ul class="nav nav-pills nav-fill" id="pills-tabs" role="tablist">
                            <li class="nav-item">
                                <a class="nav-link active" id="pills-docs-tab" data-bs-toggle="pill" data-bs-target="#pills-docs" role="tab" aria-controls="pills-docs" aria-selected="true">{"View Documents"}</a>
                            </li>
                            <li class="nav-item">
                                <a class="nav-link" id="pills-vocab-tab" data-bs-toggle="pill" data-bs-target="#pills-vocab" role="tab" aria-controls="pills-vocab" aria-selected="false">{"View Vocab"}</a>
                            </li>
                        </ul>
                        <div class="tab-content" id="pills-tabContent-userContent">
                            <div class="tab-pane fade show active pt-3" id="pills-docs" role="tabpanel" aria-labelledby="pills-docs-tab">
                                <table id="doc-table" class="table table-hover">
                                    <thead class="table-light">
                                        <tr>
                                            <th>{"Title"}</th><th>{"Source"}</th><th>{"Created On"}</th><th>{"Delete"}</th>
                                        </tr>
                                    </thead>
                                    <caption hidden=true>{"List of your saved documents."}</caption>
                                </table>
                                <button class="btn btn-outline-primary" onclick=self.link.callback(|_| Msg::DownloadDocs)>{"Export Documents as .csv"}</button>
                            </div>
                            <div class="tab-pane fade pt-3" id="pills-vocab" role="tabpanel" aria-labelledby="pills-vocab-tab">
                                <table id="vocab-table" class="table table-hover">
                                    <thead class="table-light">
                                        <tr>
                                            <th>{"Phrase"}</th><th>{"Saved From"}</th><th>{"Saved On"}</th><th>{"Radical Map"}</th><th>{"Delete"}</th>
                                        </tr>
                                    </thead>
                                    <caption hidden=true>{"List of your saved vocabulary."}</caption>
                                </table>
                                <button class="btn btn-outline-primary" onclick=self.link.callback(|_| Msg::DownloadVocab)>{"Export Vocab as .csv"}</button>
                            </div>
                        </div>
                </div>
            </div>
        </header>
        }
    }
}