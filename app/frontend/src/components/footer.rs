use crate::Route;
use yew::prelude::*;
use yew_router::prelude::*;

pub struct Footer { }

impl Component for Footer {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self { Self {  } }
    fn update(&mut self, _msg: Self::Message) -> ShouldRender { false }
    fn change(&mut self, _: Self::Properties) -> ShouldRender { false }
    fn view(&self) -> Html {
        html! {
            <div id="layoutDefault_footer">
                <footer class="footer pt-5 pb-5 mt-auto bg-light footer-light">
                    <div class="container">
                        <div class="row">
                            <div class="footer-brand mr-2">{"DuGuo"}
                                <a class="icon-list-social-link" target="_blank" href="https://github.com/ericpan64/DuGuo-Chinese-Reading-App"><i class="fab fa-github"></i></a>
                            </div>
                            <div class="mt-1">{"Designed using Start Bootstrap. \nBuilt in Rust using Rocket + Yew. \nSubmit anonymous feedback "}<RouterAnchor<Route> route=Route::Feedback>{"here."}</RouterAnchor<Route>></div>
                        </div>
                    </div>
                </footer>
            </div>
        }
    }
}