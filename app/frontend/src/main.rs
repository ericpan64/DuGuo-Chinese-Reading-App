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
    #[to="/#user/{uid}"]
    Profile(String),
    #[to="/"]
    Home,
}

struct Model {
    link: ComponentLink<Self>,
}

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
                    <a class="btn font-weight-500 ml-lg-4 btn-primary" href="#login">{"Login Now"}<img class="ml-2" src="/static/img/arrow-right.svg"/></a>
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
                            <div class="col-lg-3">
                                <div class="footer-brand">{"DuGuo"}
                                    <a class="icon-list-social-link" target="_blank" href="https://github.com/ericpan64/DuGuo-Chinese-Reading-App"><i class="fab fa-github"></i></a>
                                </div>
                                <div class="mb-3">{"Designed using Start Bootstrap. Submit anonymous feedback "}<a href="#!">{"here."}</a></div>
                            </div>
                        </div>
                        <hr class="my-5" />
                        <div class="row align-items-center">
                            <div class="col-md-6 small">{"Copyright &copy; DuGuo Maintainers 2021"}</div>
                            <div class="col-md-6 text-md-right small">
                                <a href="/#feedback">{"Feedback"}</a>
                                {"·"}
                                <a href="#!">{"Privacy Policy"}</a>
                                {"·"}
                                <a href="#!">{"Terms & Conditions"}</a>
                            </div>
                        </div>
                    </div>
                </footer>
            </div>
        }
    }
}

impl Component for Model {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self { Self { link } }
    fn update(&mut self, msg: Self::Message) -> ShouldRender { false }
    fn change(&mut self, _props: Self::Properties) -> ShouldRender { false }
    fn view(&self) -> Html {
        html! {
            <div id="layoutDefault">
                <div id="layoutDefault_content">
                    {self.view_navbar()}
                    <main>
                        <Router<AppRoute, ()> 
                            render= Router::render(|switch: AppRoute| {
                                match switch {
                                    AppRoute::Home => { html! { <Home/> }},
                                    AppRoute::Login => { html! {<Login/>}},
                                    AppRoute::Feedback => { html! {<Feedback/>}},
                                    AppRoute::Sandbox => { html! {<Sandbox/>}},
                                    AppRoute::Reader => { html! {<Reader/>}},
                                    AppRoute::Profile(uid) => { html! {<Profile uid={uid}/>}}
                                }
                            })/>
                    </main>
                    {self.view_footer()}
                </div>
            </div>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}