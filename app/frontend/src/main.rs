#![recursion_limit = "128"]
use yew::prelude::*;
use yew_router::prelude::*;
use duguo_frontend::{
    Route,
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
                        {self.view_navbar()}
                        <Router<Route, ()> 
                            render= Router::render(|switch: Route| {
                                match switch {
                                    Route::Home => { html! { <Home/> }},
                                    Route::Login => { html! { <Login/> }},
                                    Route::Feedback => { html! {<Feedback/>}},
                                    Route::Sandbox => { html! {<Sandbox/>}},
                                    Route::Reader => { html! {<Reader/>}},
                                    Route::Profile(uid) => { 
                                        html! {<Profile uid={uid}/>}
                                    }
                                }
                            })/>
                        {self.view_footer()}
                    </main>
                </div>
            </div>
        }
    }
}

// TODO: refactor these into components. A good exercise regardless
impl Model {
    fn view_navbar(&self) -> Html {
        html! {
            <nav class="navbar navbar-marketing navbar-expand-lg bg-white navbar-light fixed-top">
                <div class="container">
                    <RouterAnchor<Route> classes="navbar-brand text-primary" route=Route::Home>{"DuGuo (读国)"}</RouterAnchor<Route>>
                    <div class="collapse navbar-collapse" id="navbarSupportedContent">
                        <ul class="navbar-nav ml-auto mr-lg-5">
                            <li class="nav-item"><RouterAnchor<Route> classes="nav-link" route=Route::Home>{"Home"}</RouterAnchor<Route>></li>
                            <li class="nav-item"><RouterAnchor<Route> classes="nav-link" route=Route::Sandbox>{"Sandbox"}</RouterAnchor<Route>></li>
                            <li class="nav-item"><RouterAnchor<Route> classes="nav-link" route=Route::Feedback>{"Feedback"}</RouterAnchor<Route>></li>
                        </ul>
                        <RouterAnchor<Route> classes="btn font-weight-500 ml-lg-4 btn-primary" route=Route::Login>{"Login Now"}<img class="ml-2" src="/static/img/arrow-right.svg"/></RouterAnchor<Route>>
                    </div>
                </div>
            </nav>
        }
    }

    fn view_footer(&self) -> Html {
        html! {
            <div id="layoutDefault_footer">
                <footer class="footer pt-5 pb-5 mt-auto bg-light footer-light">
                    <div class="container">
                        <div class="row">
                            <div class="footer-brand mr-2">{"DuGuo"}
                                <a class="icon-list-social-link" target="_blank" href="https://github.com/ericpan64/DuGuo-Chinese-Reading-App"><i class="fab fa-github"></i></a>
                            </div>
                            <div class="mt-1">{"Designed using Start Bootstrap. \nBuilt using Yew + Rocket (Rust). \nSubmit anonymous feedback"}<a href="#/feedback">{" here."}</a></div>
                        </div>
                    </div>
                </footer>
            </div>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}