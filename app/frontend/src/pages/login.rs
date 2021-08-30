use yew::prelude::*;

pub struct Login {
    link: ComponentLink<Self>,
    show_pw: bool
}

pub struct UserLoginForm {
    username: String,
    password: String,
}

pub struct UserRegisterForm {
    username: String,
    email: String,
    password: String,
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

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <>
                <header class="page-header page-header-light bg-white">
                    <div class="page-header-content pt-5">
                        <div class="container">
                            <h1>{"Login / Register"}</h1>
                            <div class="alert alert-warning" role="alert">
                                {"Note that the following characters are not allowed: < > ! ( ) {{ }} \" ' ; : \\ *"}
                            </div>
                            <ul class="nav nav-pills nav-fill" id="pills-tab" role="tablist">
                                <li class="nav-item">
                                <a class="nav-link active" id="pills-login-tab" data-toggle="pill" href="#pills-login" role="tab" aria-controls="pills-login" aria-selected="true">{"Login"}</a>
                                </li>
                                <li class="nav-item">
                                <a class="nav-link" id="pills-register-tab" data-toggle="pill" href="#pills-register" role="tab" aria-controls="pills-register" aria-selected="false">{"Register"}</a>
                                </li>
                            </ul>
                            <div class="tab-content" id="pills-tabContent">
                                {self.view_login_form()}
                                {self.view_register_form()}
                            </div>
                        </div>
                    </div>
                </header>
            </>
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
    fn view_login_form(&self) -> Html {
        let flip = !&self.show_pw;
        html ! {
            <div class="tab-pane fade show active pt-3" id="pills-login" role="tabpanel" aria-labelledby="pills-login-tab">
                <form class="pt-3" id="login" onsubmit=self.link.callback(|_| Msg::AttemptLogin)>
                    <div class="form-group">
                        <input class="form-control" type="text" name="username" placeholder="Username" required=true/>
                    </div>
                    <div class="form-group">
                        <input id="pw-login" class="form-control" type=self.get_pw_type() name="password" placeholder="Password" required=true/>
                    </div>
                    <div class="form-check">
                        <input type="checkbox" class="form-check-input" id="showPwCheck"/>
                        <label class="form-check-label" for="showPwCheck" onclick=self.link.callback(move |_| Msg::ShowPassword(flip))>{"Show Password"}</label>
                    </div>
                    <button class="btn btn-outline-primary" type="submit">{"Log In"}</button>
                </form>
            </div>
        }
    }
    fn view_register_form(&self) -> Html {
        let flip = !&self.show_pw;
        html! {
            <div class="tab-pane fade pt-3" id="pills-register" role="tabpanel" aria-labelledby="pills-register-tab">
                <form class="pt-3" id="register" onsubmit=self.link.callback(|_| Msg::AttemptRegister)>
                    <div class="form-group">
                        <input class="form-control" type="text" name="username" placeholder="Username" required=true/>
                    </div>
                    <div class="form-group">
                        <input id="pw-reg" class="form-control" type=self.get_pw_type() name="password" placeholder="Password (min 8 chars)" minlength="8" required=true/>
                    </div>
                    <div class="form-group">
                        <input class="form-control" type="email" name="email" placeholder="Email" required=true/>
                    </div>
                    <div class="form-check">
                        <input type="checkbox" class="form-check-input" id="showPwCheck"/>
                        <label class="form-check-label" for="showPwCheck" onclick=self.link.callback(move |_| Msg::ShowPassword(flip))>{"Show Password"}</label>
                    </div>
                    <button class="btn btn-outline-primary" type="submit">{"Register"}</button>
                </form>
            </div>
        }
    }
}
