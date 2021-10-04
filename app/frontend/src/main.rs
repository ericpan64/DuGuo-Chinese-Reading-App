#![recursion_limit = "128"]
use yew::prelude::*;
use yew_router::prelude::*;
use duguo_frontend::pages::*;

#[derive(Switch, Clone, Debug)]
pub enum AppRoute {
    #[to="/#login"]
    Login,
    #[to="/#feedback"]
    Feedback,
    #[to="/#sandbox"]
    Sandbox,
    #[to="/#reader"]
    Reader,
    #[to="/#u/{uid}"]
    Profile(String),
    #[to="/"]
    Home,
}

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
                        <Router<AppRoute, ()> 
                            render= Router::render(|switch: AppRoute| {
                                match switch {
                                    AppRoute::Home => { html! { <Home/> }},
                                    AppRoute::Login => { html! { <Login/> }},
                                    AppRoute::Feedback => { html! {<Feedback/>}},
                                    AppRoute::Sandbox => { html! {<Sandbox/>}},
                                    AppRoute::Reader => { html! {<Reader/>}},
                                    AppRoute::Profile(uid) => { 
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
                <a class="navbar-brand text-primary" href="/">{"DuGuo (读国)"}</a><button class="navbar-toggler" type="button" data-toggle="collapse" data-target="#navbarSupportedContent" aria-controls="navbarSupportedContent" aria-expanded="false" aria-label="Toggle navigation"><i data-feather="menu"></i></button>
                <div class="collapse navbar-collapse" id="navbarSupportedContent">
                    <ul class="navbar-nav ml-auto mr-lg-5">
                        <li class="nav-item"><a class="nav-link" href="/">{"Home"}</a></li>
                        <li class="nav-item"><a class="nav-link" href="/#sandbox">{"Sandbox"}</a></li>
                        <li class="nav-item"><a class="nav-link" href="/#feedback">{"Feedback"}</a></li>
                    </ul>
                    <a class="btn font-weight-500 ml-lg-4 btn-primary" href="/#login">{"Login Now"}<img class="ml-2" src="/static/img/arrow-right.svg"/></a>
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