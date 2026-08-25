use dioxus::prelude::*;
use shared::{User, PasswordResetRequest, TokenCheckRequest};
use crate::{Route, services::api};

#[component]
pub fn UserPasswordResetPage(token: String) -> Element {
    let nav = navigator();

    if token.is_empty() {
        nav.replace(Route::HomePage {});
        return rsx! { "Invalid token" };
    }

    // User loaded from token
    let user = use_signal(|| None::<User>);

    // Parameter fields + error
    let token = use_signal(|| token.clone());
    let mut new_password = use_signal(|| String::new());
    let mut new_password2 = use_signal(|| String::new());
    let api_error = use_signal(|| String::new());

    // Run token check once
    use_effect(move || {
        let mut user = user.clone();
        
        spawn(async move {
            let token = token.clone().to_string();
            let query = TokenCheckRequest { token };
            let result = api::check_reset_token(query).await;
            user.set(result); // Option<User>
        });

        //(||) {}
    });

    let passwords_mismatch = move || {
        !new_password2.read().is_empty()
            && *new_password.read() != *new_password2.read()
    };

    let confirm_class = move || {
        if passwords_mismatch() { "error" } else { "" }
    };

    let result = user.read().clone();

    match result {
        
        None => {
            rsx!(
                div {
                    h1 { "Invalid or expired reset link" }
                }
            )
        }
        Some(u) => {
            rsx!(
                div { id: "change_password",
                    h1 { "Please enter your new password" }

                    div { id: "input-rows",
                        table {
                            tbody {
                                tr {
                                    td {
                                        label { "New Password :" }
                                    }
                                    td {
                                        input {
                                            r#type: "password",
                                            value: "{new_password}",
                                            oninput: move |e: Event<FormData>| { new_password.set(e.value().to_string()) },
                                        }
                                    }
                                }

                                tr {
                                    td {
                                        label { "Re-type New Password :" }
                                    }
                                    td {
                                        input {
                                            r#type: "password",
                                            value: "{new_password2}",
                                            class: "{confirm_class()}",
                                            oninput: move |e: Event<FormData>| { new_password2.set(e.value().to_string()) },
                                        }

                                        if passwords_mismatch() {
                                            div { class: "input-error", "New passwords do not match." }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if !passwords_mismatch() {
                        div {
                            button {
                                id: "page-button",
                                onclick: {
                                    let new_password = new_password.clone();
                                    let mut api_error = api_error.clone();
                                    let token = token.clone();

                                    move |_| {
                                        let nav = navigator();
                                        let user = u.clone();

                                        spawn(async move {
                                            let req = PasswordResetRequest {
                                                token: token.to_string(),
                                                user,
                                                new_password: new_password.read().clone(),
                                            };

                                            match api::reset_user_password(req).await {
                                                Ok(true) => {
                                                    api_error.set(String::new());
                                                    nav.push(Route::LoginPage {});
                                                }
                                                Ok(false) => {
                                                    api_error.set("Password reset failed".into());
                                                }
                                                Err(err) => {
                                                    api_error.set(err);
                                                }
                                            }
                                        });
                                    }
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
            )
        }
    }
}
