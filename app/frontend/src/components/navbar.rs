use yew::prelude::*;
use yew_router::prelude::*;
use super::super::Route;

pub struct Navbar { }

impl Component for Navbar {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self { Self {  } }
    fn update(&mut self, _msg: Self::Message) -> ShouldRender { false }
    fn change(&mut self, _: Self::Properties) -> ShouldRender { false }
    fn view(&self) -> Html {
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
}