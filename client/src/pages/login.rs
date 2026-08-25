use dioxus::prelude::*;
use shared::LoginRequest;
use crate::{Route, services::api, state::auth::AUTH};

#[component]
pub fn LoginPage() -> Element {
    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());

    rsx! {
        div { id: "login",
            //h1 { "Login" }
            div { id: "input-rows",
                table {
                    tbody {
                        tr {
                            td {
                                label { "Email : " }
                            }
                            td {
                                input {
                                    r#type: "email",
                                    value: "{email}",
                                    oninput: move |e: Event<FormData>| email.set(e.value().to_string()),
                                }
                            }
                        }

                        tr {
                            td {
                                label { "Password : " }
                            }
                            td {
                                input {
                                    r#type: "password",
                                    value: "{password}",
                                    oninput: move |e: Event<FormData>| password.set(e.value().to_string()),
                                }
                            }
                        }
                    
                    }
                }
            }

            div {
                button {
                    id: "page-button",
                    onclick: {
                        let email = email.clone();
                        let password = password.clone();

                        move |_| {
                            let nav = navigator();

                            spawn(async move {
                                let req = LoginRequest {
                                    email: email.read().clone(),
                                    password: password.read().clone(),
                                };
                                log::debug!("Request login for : {:?}", req.email);
                                if let Some(user) = api::login(req).await {
                                    AUTH.write().user = Some(user);
                                    // Check if the user requires a password reset
                                    if AUTH.read().requires_password_change() {
                                        log::debug!("Calling change password page");
                                        nav.push(Route::UserPasswordChangePage {});
                                    } else {
                                        nav.push(Route::HomePage {});
                                    }
                                }
                            });
                        }
                    },
                    "Login"
                }

                button {
                    id: "page-button",
                    onclick: move |_| {
                        let nav = navigator();
                        nav.push(Route::UserForgotPasswordPage {});
                    },
                    "Forgot Password"
                }
            
            }
        }
    }
}
 