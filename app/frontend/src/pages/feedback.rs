use yew::prelude::*;

pub struct Feedback {
    link: ComponentLink<Self>,
}

pub enum Msg {
    SubmitForm,
}

impl Component for Feedback {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            SubmitForm => { 
                false
            },
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

impl Feedback {
    fn view_header(&self) -> Html {
        html! {
            <header class="page-header page-header-light bg-white">
                <div class="page-header-content">
                    <div class="container">
                        <h1>{"Feedback"}</h1>
                        <p>{"Let us know your thoughts below! Leave your contact info (optional) if you would like a response and/or are open to further discuss your thoughts."}</p>
                        <p>{"To report more systemic bugs, please open a "}<a target="_blank" href="https://github.com/ericpan64/DuGuo-Chinese-Reading-App/issues">{"Github Issue."}</a></p>
                        <textarea name="feedback" form="upload" rows="10" cols="100" required=true>{"Add your feedback here!"}</textarea>
                        <form id="upload" onsubmit=self.link.callback(|_| Msg::SubmitForm) method="POST">
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
