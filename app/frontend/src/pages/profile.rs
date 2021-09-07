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

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <>
                {self.view_header()}
            </>
        }
    }
}

impl Profile {
    fn view_header(&self) -> Html {
        html! {
            <header class="page-header page-header-light bg-white">
                <div class="page-header-content">
                    <div class="container">
                        <h1>{self.uid.as_str()}</h1>
                            <hr/>
                            <h5>{"Settings"}</h5>
                                <button class="btn btn-primary dropdown-toggle" id="phonetic-setting" type="button" data-bs-toggle="dropdown" aria-expanded="false">
                                    {self.phonetic_type.as_str()}
                                </button>
                                <ul class="dropdown-menu" aria-labelledby="dropdownMenuButton">
                                    <li><a class="dropdown-item" onclick=self.link.callback(|_| Msg::UpdatePhonetic(Phonetic::Pinyin))>{"View Pinyin"}</a></li>
                                    <li><a class="dropdown-item" onclick=self.link.callback(|_| Msg::UpdatePhonetic(Phonetic::Zhuyin))>{"View Zhuyin (Bopomofo)"}</a></li>
                                </ul>
                                <button class="btn btn-primary dropdown-toggle" id="char-setting" type="button" data-bs-toggle="dropdown" aria-expanded="false">
                                    {self.cn_type.as_str()}
                                </button>
                                <ul class="dropdown-menu" aria-labelledby="dropdownMenuButton">
                                    <li><a class="dropdown-item" onclick=self.link.callback(|_| Msg::UpdateCnType(CnType::Simplified))>{"View Simplified"}</a></li>
                                    <li><a class="dropdown-item" onclick=self.link.callback(|_| Msg::UpdateCnType(CnType::Traditional))>{"View Traditional"}</a></li>
                                </ul>
                                <br/><br/>
                                <p>{"FYI: Saved Documents + Vocab are linked to the Chinese type+phonetic combination. So updating these settings will cause the page to refresh."}</p>
                            <br/>
                                <h4>{"Upload Document"}</h4>
                                <hr/>
                                <ul class="nav nav-pills nav-fill" id="pills-tab" role="tablist">
                                    <li class="nav-item">
                                        <a class="nav-link" id="pills-url-tab" data-toggle="pill" href="#pills-url" role="tab" aria-controls="pills-vocab" aria-selected="false">{"Parse from URL"}</a>
                                    </li>
                                    <li class="nav-item">
                                    <a class="nav-link active" id="pills-cp-tab" data-toggle="pill" href="#pills-cp" role="tab" aria-controls="pills-docs" aria-selected="true">{"Copy & Paste"}</a>
                                    </li>
                                    <div class="tab-content" id="pills-tabContent-userContent">
                                        <div class="tab-pane fade show active pt-3" id="pills-url" role="tabpanel" aria-labelledby="pills-url-tab">
                                            <h5>{"Parse from URL"}</h5>
                                            <form class="form" action="/api/url-upload" id="user-url-form" onsubmit=self.link.callback(|_| Msg::SwitchToLoadingButton) method="POST">
                                                <input type="text" name="url" placeholder="Document URL" required=true/>
                                                <button id="url-upload-button" class="btn btn-outline-primary" type="submit">{"Upload URL"}</button>
                                            </form>
                                        </div>
                                        <div class="tab-pane fade pt-3" id="pills-cp" role="tabpanel" aria-labelledby="pills-cp-tab">
                                            <h5>{"Copy & Paste"}</h5>
                                            <form class="form" action="/api/upload" id="user-doc-form" onsubmit=self.link.callback(|_| Msg::SwitchToLoadingButton) method="POST">
                                                <input type="text" name="title" placeholder="Document Title" required=true/><br/>
                                                <input type="text" name="source" placeholder="Document Source (optional)"/>
                                                <button id="doc-upload-button" class="btn btn-outline-primary" type="submit">{"Upload Text"}</button>
                                                <textarea class="form-control" name="body" form="user-doc-form" rows="2" cols="35" required=true></textarea>
                                            </form>
                                        </div>
                                    </div>
                                </ul>
                                <h4>{"View Documents + Vocab"}</h4>
                                <hr/>
                                <ul class="nav nav-pills nav-fill" id="pills-tab-userContent" role="tablist">
                                    <li class="nav-item">
                                    <a class="nav-link active" id="pills-docs-tab" data-toggle="pill" href="#pills-docs" role="tab" aria-controls="pills-docs" aria-selected="true">{"View Documents"}</a>
                                    </li>
                                    <li class="nav-item">
                                    <a class="nav-link" id="pills-vocab-tab" data-toggle="pill" href="#pills-vocab" role="tab" aria-controls="pills-vocab" aria-selected="false">{"View Vocab"}</a>
                                    </li>
                                </ul>
                            
                        <div class="tab-content" id="pills-tabContent-userContent">
                            <div class="tab-pane fade show active pt-3" id="pills-docs" role="tabpanel" aria-labelledby="pills-docs-tab">
                                {"{{ doc_table | safe }}"}
                                {"# TODO: make this a modal popup w/ option to disable in future #"}
                                {"# <p><strong>*Note*</strong>: Documents are linked to Saved Vocab, so deleting a Document also deletes Vocab saved from that Document.</p> #"}
                                <button class="btn btn-outline-primary" onclick=self.link.callback(|_| Msg::DownloadDocs)>{"Export Documents as .csv"}</button>
                            </div>
                            <div class="tab-pane fade pt-3" id="pills-vocab" role="tabpanel" aria-labelledby="pills-vocab-tab">
                                {"{{ vocab_table | safe }}"}
                                <button class="btn btn-outline-primary" onclick=self.link.callback(|_| Msg::DownloadVocab)>{"Export Vocab as .csv"}</button>
                            </div>
                        </div>
                    </div>
                </div>
            </header>
        }
    }

}
