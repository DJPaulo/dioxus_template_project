use dioxus::prelude::*;
use shared::ForgotPasswordRequest;
use crate::services::api;  //Route

#[component]
pub fn UserForgotPasswordPage() -> Element {

    let mut email = use_signal(|| String::new());
    let api_error = use_signal(|| String::new());

    rsx! {
        div { id: "change_password",
            h1 { "Reset password" }
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
                    
                    }
                }
            }

            div {
                button {
                    id: "page-button",
                    onclick: move |_| {
                        //let nav = navigator();
                        let email = email.clone();
                        let mut api_error = api_error.clone();

                        spawn(async move {
                            let email = email.read().clone();
                            let req = ForgotPasswordRequest {
                                email: email,
                            };
                            let msg = "If this is a valid user, a reset link has been emailed."
                                .to_string();
                            match api::user_request_user_password_reset(req).await {
                                Ok(_) | Err(_) => api_error.set(msg),
                            }
                        });
                    },
                    "Submit"
                }
            }
            if !api_error.read().is_empty() {
                div { class: "api-error",
                    h2 { "{api_error.read()}" }
                }
            }
        
        }
    }
}
