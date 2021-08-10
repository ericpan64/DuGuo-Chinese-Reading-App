use yew::prelude::*;
use duguo_frontend::app_router::AppRouter;

enum Msg {
    AddOne,
    SubtractOne,
}

struct Model {
    link: ComponentLink<Self>,
    value: i64,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            value: 0,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::AddOne => {
                self.value += 1;
                true // prompts re-render, I think?
            },
            Msg::SubtractOne => {
                self.value -= 1;
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Returns "true" if props differ
        // In this case, there are no props (yet...)
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <AppRouter/>
                <button onclick=self.link.callback(|_| Msg::AddOne)>{"+1"}</button>
                <p>{ self.value }</p>
                <button onclick=self.link.callback(|_| Msg::SubtractOne)>{"-1"}</button>
            </div>
        }
    }
}

/// Testing Vim editing to start
fn main() {
    yew::start_app::<Model>();
}