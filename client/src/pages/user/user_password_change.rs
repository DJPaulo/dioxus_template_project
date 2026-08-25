use dioxus::prelude::*;
use shared::ChangePasswordRequest;
use crate::{Route, services::api, state::auth::AUTH};

#[component]
pub fn UserPasswordChangePage() -> Element {
    let auth = AUTH.read();

    let mut old_password = use_signal(|| String::new());
    let mut new_password = use_signal(|| String::new());
    let mut new_password2 = use_signal(|| String::new());
    let api_error = use_signal(|| String::new());

    let passwords_mismatch = move || {
        !new_password2.read().is_empty() && *new_password.read() != *new_password2.read()
    };

    let confirm_class = move || {
        if passwords_mismatch() {
            "error"
        } else {
            ""
        }
    };

    let user = match &auth.user {
        Some(user) => user.clone(),
        None => {
            // handle missing user
            return rsx!(
                div {
                    h1 { "Not logged in" }
                }
                // handle missing user
            );
        }
    };

    rsx! {
        div { id: "change_password",
            h1 { "Please change your password" }
            div { id: "input-rows",
                table {
                    tbody {

                        tr {
                            td {
                                label { "Old Password : " }
                            }
                            td {
                                input {
                                    r#type: "password",
                                    value: "{old_password}",
                                    oninput: move |e: Event<FormData>| old_password.set(e.value().to_string()),
                                }
                            }
                        }

                        tr {
                            td {
                                label { "New Password : " }
                            }
                            td {
                                input {
                                    r#type: "password",
                                    value: "{new_password}",
                                    oninput: move |e: Event<FormData>| new_password.set(e.value().to_string()),
                                }
                            
                            }
                        
                        }

                        tr {
                            td {
                                label { "Re-type New Password : " }
                            }
                            td {
                                input {
                                    r#type: "password",
                                    value: "{new_password2}",
                                    class: "{confirm_class()}",
                                    oninput: move |e: Event<FormData>| new_password2.set(e.value().to_string()),
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

                            let old_password = old_password.clone();
                            let new_password = new_password.clone();
                            let mut api_error = api_error.clone();

                            move |_| {
                                let nav = navigator();
                                let user = user.clone();
                                spawn(async move {
                                    let req = ChangePasswordRequest {
                                        user: user,
                                        old_password: old_password.read().clone(),
                                        new_password: new_password.read().clone(),
                                    };
                                    log::debug!("Request password change for : {:?}", req.user);
                                    match api::change_user_password(&req).await {
                                        Ok(true) => {
                                            log::debug!("Change successful");
                                            api_error.set(String::new());
                                            nav.push(Route::LoginPage {});
                                        }
                                        Ok(false) => {
                                            log::debug!("Change password failed");
                                            api_error.set("Password update failed".into());
                                        }
                                        Err(err) => {
                                            log::debug!("Change error: {:?}", err);
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
    }
}
