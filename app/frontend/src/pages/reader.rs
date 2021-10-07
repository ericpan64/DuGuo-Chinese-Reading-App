use anyhow::Error;
use crate::{
    CnPhrase,
    CnPhonetics,
    CnType,
    SandboxDoc,
    components::PhraseSpan
};
use yew::format::{Json, Nothing};
use yew::services::{
    console::ConsoleService,
    fetch::{FetchService, FetchTask, Request, Response}
};
use yew::prelude::*;

pub struct Reader {
    link: ComponentLink<Self>,
    phrase_list: Vec<CnPhrase>,
    cn_type: CnType,
    cn_phonetics: CnPhonetics,
    ft: FetchTask
}

pub enum Msg {
    DataReceived(Result<SandboxDoc, Error>),
    HideSavedPhonetics,
    HideAllPhonetics,
    ShowAllPhonetics,
    StartReader,
    ResetReader,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub doc_id: String,
}

impl Component for Reader {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self { 
        let doc_id = props.doc_id;
        let req = Request::get(format!("/api/get-doc/{}", &doc_id))
            .body(Nothing)
            .unwrap();
        let callback = link.callback(|response: Response<Json<Result<SandboxDoc, Error>>>| {
            let (_meta, Json(data)) = response.into_parts();
            Msg::DataReceived(data)
        });
        let ft = FetchService::fetch(req, callback).unwrap();
        // Set as placeholders
        let phrase_list = Vec::<CnPhrase>::new();
        let cn_type = CnType::Simplified;
        let cn_phonetics = CnPhonetics::Pinyin;
        Self { link, ft, phrase_list, cn_type, cn_phonetics }
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        // TODO: implement phonetic showing
        // TODO: implement Reader start/stop
        match msg {
            Msg::DataReceived(data) => {
                ConsoleService::log(&format!("DATA: {:?}", data));
                let doc: SandboxDoc = data.unwrap();
                self.phrase_list = doc.tokenized_body_json;
                self.cn_type = doc.cn_type;
                self.cn_phonetics = doc.cn_phonetics;
                true
            }
            _ => false
        }
    }
    fn change(&mut self, _: Self::Properties) -> ShouldRender { false }
    fn view(&self) -> Html {
        html! {
            <header class="page-header page-header-light bg-white">
                <div class="page-header-content">
                    <div class="container">
                        <button class="btn btn-primary" type="button" data-bs-toggle="collapse" data-bs-target="#instructions">{"Instructions"}</button>
                        <div id="instructions" class="collapse">
                            <div class="card card-body">
                                <ul>
                                    <li>{"Click on a phrase to view more information, speech-to-text, and save it to your dictionary."}</li>
                                    <li>{"Tab to move to the next phrase, Shift + Tab to move to the previous phrase."}</li>
                                    <li>{"Press r to Start/Stop the Text-to-Speech reading."}</li>
                                    <li>{"Press any key, click, or scroll to close all active pop-ups."}</li>
                                    <li>{"Use the buttons to toggle phonetics settings."}</li>
                                </ul>
                            </div>
                        </div>
                        <br/><br/>
                        <div class="btn-group" role="group" aria-label="Settings for phonetics visibility.">
                            <button id="hide-saved" class="btn btn-primary border border-light" onclick=self.link.callback(|_| Msg::HideSavedPhonetics)>{"Hide Saved Phonetics"}</button>
                            <button id="hide-all" class="btn btn-primary border border-light" onclick=self.link.callback(|_| Msg::HideAllPhonetics)>{"Hide All Phonetics"}</button>
                            <button id="reset-all" class="btn btn-primary border border-light" onclick=self.link.callback(|_| Msg::ShowAllPhonetics)>{"Show All Phonetics"}</button>
                        </div>
                        <div id="reader-btn-group" class="btn-group" role="group" aria-label="Settings for document reader.">
                            <button id="read-start-stop" class="btn btn-primary border border-light" onclick=self.link.callback(|_| Msg::StartReader)>{"Read Document Aloud"}</button>
                            <button id="read-reset" class="btn btn-primary border border-light" onclick=self.link.callback(|_| Msg::ResetReader)>{"Reset Reader Position"}</button>
                        </div>
                    </div>
                    <div class="container pt-5">
                        { for self.phrase_list.iter().map(|p| Self::generate_phrase_span(p.clone(), false, self.cn_type.clone(), self.cn_phonetics.clone())) }
                    </div>
                </div>
            </header>
        }
    }
}

impl Reader {
    fn generate_phrase_span(p: CnPhrase, has_learned: bool, cn_type: CnType, cn_phonetics: CnPhonetics) -> Html {
        html! { 
            <PhraseSpan phrase=p has_learned=has_learned cn_type=cn_type cn_phonetics=cn_phonetics /> 
        }
    }
}