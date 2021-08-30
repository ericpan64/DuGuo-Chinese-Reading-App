use yew::prelude::*;
use yew::html::IntoPropValue;
use std::borrow::Cow;

pub struct Home {
    link: ComponentLink<Self>,
    duey_img: String,
}

pub enum Msg {
    NormalBaseDuey,
    NormalDuey,
    SurprisedDuey
}

impl Component for Home {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let duey_img = String::from("static/img/duey/duey_normal.png");
        Self { link, duey_img }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::NormalBaseDuey => {
                self.duey_img = String::from("static/img/duey/duey_base_normal.png");
                true
            },
            Msg::NormalDuey => {
                self.duey_img = String::from("static/img/duey/duey_normal.png");
                true
            }
            Msg::SurprisedDuey => {
                self.duey_img = String::from("static/img/duey/duey_surprised.png");
                true
            }
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <>
                {self.view_header()}
                {self.view_stack()}
                {self.view_duey()}
            </>
        }
    }
}

impl Home {
    fn view_header(&self) -> Html {
        html! {
            <header class="page-header page-header-light bg-white">
                <div class="page-header-content pt-5">
                    <div class="container">
                        <div class="row align-items-center">
                            <div class="col-lg-6" data-aos="fade-up">
                                <h1 class="page-header-title">{"Learn how to read Chinese the right way"}</h1>
                                <p class="page-header-text mb-5">{"Welcome to DuGuo, an open-source web app for learning Chinese reading. Pick content you care about, measure your progress, and say 再见 to outdated 课本!"}</p>
                                <a class="btn btn-lg btn-primary font-weight-500 mr-3" href="#sandbox">{"Try the Sandbox"}<i class="ml-2" data-feather="arrow-right"></i></a><a class="btn btn-lg btn-primary-soft text-primary font-weight-500" href="#docs">{"Documentation"}</a>
                            </div>
                            <div class="col-lg-6 d-none d-lg-block" data-aos="fade-up" data-aos-delay="100"><img class="img-fluid" src="static/img/Insertion-sort-example.gif" /></div>
                        </div>
                    </div>
                </div>
                <div class="svg-border-rounded text-light">
                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 144.54 17.34" preserveAspectRatio="none" fill="currentColor"><path d="M144.54,17.34H0V0H144.54ZM0,0S32.36,17.34,72.27,17.34,144.54,0,144.54,0"></path></svg>
                </div>
            </header>
        }
    }

    fn view_stack(&self) -> Html {
        html! { 
            <section class="bg-light py-10">
                <div class="container">
                    <div class="row text-center">
                        <div class="col-lg-4 mb-5 mb-lg-0">
                            <div class="icon-stack icon-stack-xl bg-gradient-primary-to-secondary text-white mb-4"><img src="/static/img/book-open.svg"/></div>
                            <h3>{"Read what you want"}</h3>
                            <p class="mb-0">{"Upload arbitrary Chinese text into a context-rich learning environment. Get difficulty estimates based on the HSK framework."}</p>
                        </div>
                        <div class="col-lg-4 mb-5 mb-lg-0">
                            <div class="icon-stack icon-stack-xl bg-gradient-primary-to-secondary text-white mb-4"><img src="/static/img/layers.svg"/></div>
                            <h3>{"Track your progress"}</h3>
                            <p class="mb-0">{"Save learned vocabulary as you go. Easily export your data for learning with external platforms like Anki."}</p>
                        </div>
                        <div class="col-lg-4">
                            <div class="icon-stack icon-stack-xl bg-gradient-primary-to-secondary text-white mb-4"><img src="/static/img/code.svg"/></div>
                            <h3>{"Modify the code"}</h3>
                            <p class="mb-0">{"This project is open-source and available on GitHub ("}<a href="https://github.com/ericpan64/DuGuo-Chinese-Reading-App" target = "_blank">{"link"}</a>{"). Issues, forks, and PRs welcome!"}</p>
                        </div>
                    </div>
                </div>
                <div class="svg-border-rounded text-white">
                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 144.54 17.34" preserveAspectRatio="none" fill="currentColor"><path d="M144.54,17.34H0V0H144.54ZM0,0S32.36,17.34,72.27,17.34,144.54,0,144.54,0"></path></svg>
                </div>
            </section>
        }
    }

    fn view_duey(&self) -> Html {
        html! {
            <div class="bg-white py-5">
                <div class="container">
                    <div class="row justify-content-center">
                        <div class="col-lg-8">
                            <div class="text-center mb-5" data-aos="fade-up">
                                <h2>{"Learn with your friends, including Duey!"}</h2>
                                <img width="200em" src={self.duey_img.clone()} 
                                    onmouseover=self.link.callback(|_| Msg::NormalDuey)
                                    onmouseout=self.link.callback(|_| Msg::NormalBaseDuey)
                                    alt="Duey, the DuGuo mascot!"/>
                                <p class="lead">{"Share source links with your friends to sync collaboration and learning. In the meanwhile, Duey (对龙) is here to keep you company and cheer you on!"}</p>
                                <p onmouseover=self.link.callback(|_| Msg::SurprisedDuey)
                                    onmouseout=self.link.callback(|_| Msg::NormalBaseDuey)>{"(and he'll leave you alone if you ask him to)"}</p>
                            </div>
                            <div class="list-group small mb-2">
                                <a class="btn btn-lg btn-primary font-weight-500" href="#sandbox">{"Try it now!"}<i class="ml-2" data-feather="arrow-right"></i></a>
                                <br/>
                                <a class="btn btn-lg btn-primary-soft text-primary font-weight-500" href="#login">{"Create an Account"}</a>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}
