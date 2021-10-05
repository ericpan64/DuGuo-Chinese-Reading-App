use yew::prelude::*;

pub struct Login {
    link: ComponentLink<Self>,
    show_pw: bool
}

pub enum Msg {
    AttemptLogin,
    AttemptRegister,
    ShowPassword(bool)
}

impl Component for Login {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let show_pw = false;
        Self { link, show_pw }
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        // TODO: implement these functions to perform API request
        match msg {
            Msg::AttemptLogin => { 
                false
            },
            Msg::AttemptRegister => {
                false
            },
            Msg::ShowPassword(b) => {
                self.show_pw = b;
                true
            }
        }
    }
    fn change(&mut self, _: Self::Properties) -> ShouldRender { false }
    fn view(&self) -> Html {
        let flip = !&self.show_pw;
        html! {
            <div class="page-header-content">
                <header class="page-header page-header-light bg-white">
                    <div class="container">
                        <h1>{"Login / Register"}</h1>
                        <div class="alert alert-warning" role="alert">
                            {"Note that the following characters are not allowed: < > ! ( ) { } \" ' ; : \\ *"}
                        </div>
                        <div class="container">
                            <form class="pt-3" id="form">
                                <div class="form-group">
                                    <input class="form-control" type="text" name="username" placeholder="Username" required=true/>
                                </div>
                                <div class="form-group">
                                    <input id="pw-reg" class="form-control" type=self.get_pw_type() name="password" placeholder="Password (min 8 chars)" minlength="8" required=true/>
                                </div>
                                <div class="form-group">
                                    <input class="form-control" type="email" name="email" placeholder="Email (optional)"/>
                                </div>
                                <div class="form-check mb-2">
                                    <input type="checkbox" class="form-check-input" id="showPwCheck"/>
                                    <label class="form-check-label" for="showPwCheck" onclick=self.link.callback(move |_| Msg::ShowPassword(flip))>{"Show Password"}</label>
                                </div>
                                <button class="btn btn-primary mr-1" onclick=self.link.callback(|_| Msg::AttemptLogin)>{"Login"}</button>
                                <button class="btn btn-outline-primary ml-1" onclick=self.link.callback(|_| Msg::AttemptRegister)>{"Register"}</button>
                            </form>
                        </div>
                    </div>
                </header>
            </div>
        }
    }
}

impl Login {
    fn get_pw_type(&self) -> String {
        let mut res = String::with_capacity(10);
        if self.show_pw {
            res += "text";
        } else {
            res += "password";
        }
        return res;
    }
}