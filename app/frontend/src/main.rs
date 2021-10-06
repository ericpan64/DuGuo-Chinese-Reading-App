#![recursion_limit = "128"]
use yew::prelude::*;
use yew_router::prelude::*;
use duguo_frontend::{
    Route,
    components::*,
    pages::*
};

struct Model { }

impl Component for Model {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self { Self { } }
    fn update(&mut self, _msg: Self::Message) -> ShouldRender { false }
    fn change(&mut self, _props: Self::Properties) -> ShouldRender { false }
    fn view(&self) -> Html {
        html! {
            <div id="layoutDefault">
                <div id="layoutDefault_content">
                    <main>
                        <Navbar/>
                        <Router<Route, ()> 
                            render= Router::render(|switch: Route| {
                                match switch {
                                    Route::Home => { html! { <Home/> }},
                                    Route::Login => { html! { <Login/> }},
                                    Route::Feedback => { html! {<Feedback/>}},
                                    Route::Sandbox => { html! {<Sandbox/>}},
                                    Route::Reader(uid) => { html! {<Reader uid={uid} />}},
                                    Route::Profile(uid) => { 
                                        html! {<Profile uid={uid}/>}
                                    }
                                }
                            })/>
                        <Footer/>
                    </main>
                </div>
            </div>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}