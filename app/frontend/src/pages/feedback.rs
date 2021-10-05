
use yew::prelude::*;
use yew::services::{ConsoleService, DialogService};

pub struct Feedback {
    link: ComponentLink<Self>,
}

impl Component for Feedback {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self { Self { link } }
    fn update(&mut self, _msg: Self::Message) -> ShouldRender { false }
    fn change(&mut self, _: Self::Properties) -> ShouldRender { false }
    fn view(&self) -> Html {
        let form_callback = self.link.callback(|e: FocusEvent| {
            // TODO: implement this to perform appropriate API request to /api/feedback
            println!("{:?}", e);
            let event_str = format!("Event: {:?}", e);
            ConsoleService::info(&event_str);
            DialogService::alert(&event_str);
            // FocusEvent { obj: UiEvent { obj: Event { obj: Object { obj: JsValue(SubmitEvent) } } } }

            // Make reqwest
        });
        html! {
            <header class="page-header page-header-light bg-white">
                <div class="page-header-content">
                    <div class="container">
                        <h1>{"Feedback"}</h1>
                        <p>{"Let us know your thoughts below! Leave your contact info (optional) if you would like a response and/or are open to further discuss your thoughts."}</p>
                        <p>{"To report more systemic bugs, please open a "}<a target="_blank" href="https://github.com/ericpan64/DuGuo-Chinese-Reading-App/issues">{"Github Issue."}</a></p>
                        <textarea name="feedback" form="upload" rows="10" cols="100" required=true>{"Add your feedback here!"}</textarea>
                        <form action="/api/feedback" id="upload" onsubmit=form_callback method="POST">
                            <div class="form-group">
                                <input type="text" name="contact" placeholder="Contact (optional)" />
                            </div>
                            <button class="btn btn-outline-primary" type="submit">{"Submit"}</button>
                        </form>
                    </div>
                </div>
            </header>
        }
    }
}