use dioxus::prelude::*;
use shared::Role;
use crate::{Route, services::api};
use crate::state::{auth::AUTH, user_edit::EDITING_USER};

#[component]
pub fn UserAddEditPage() -> Element {
    let auth = AUTH.read();
    let nav = navigator();

    if auth.user.is_none() {
        nav.replace(Route::LoginPage {});
        return rsx! { "Forbidden" };
    }

    if !auth.is_admin() {
        nav.replace(Route::HomePage {});
        return rsx! { "Forbidden" };
    }

    if auth.requires_password_change() {
        nav.replace(Route::UserPasswordChangePage {});
        return rsx! { "Password change required" };
    }

    let editing_user = EDITING_USER.read();
    let initial_id = editing_user.user.as_ref().map(|u| u.id).unwrap_or_default();
    let initial_username = editing_user.user.as_ref().map(|u| u.username.clone()).unwrap_or_default();
    let initial_email = editing_user.user.as_ref().map(|u| u.email.clone()).unwrap_or_default();
    let initial_role = editing_user.user.as_ref().map(|u| match u.role {
        Role::Admin => "Admin".to_string(),
        Role::User => "User".to_string(),
    }).unwrap_or_else(|| "User".to_string());
    let initial_is_active = editing_user.user.as_ref().map(|u| u.is_active).unwrap_or(true);
    let initial_change_password = editing_user.user.as_ref().map(|u| u.change_password).unwrap_or(true);
    drop(editing_user);

    let user_id = use_signal(|| initial_id);
    let mut username = use_signal(|| initial_username);
    let mut email = use_signal(|| initial_email);
    let mut role = use_signal(|| initial_role);
    let mut is_active = use_signal(|| initial_is_active);
    let change_password = use_signal(|| initial_change_password);

    rsx! {
        div {
            h1 {
                if EDITING_USER.read().user.is_some() {
                    "Edit User"
                } else {
                    "Add New User"
                }
            }

            div { id: "input-rows",
                table {
                    tbody {
                        tr {
                            td {
                                label { "Email : " }
                            }
                            td {
                                if EDITING_USER.read().user.is_some() {
                                    input {
                                        r#type: "email",
                                        value: "{email}",
                                        readonly: true, // Prevent editing email for existing users
                                    }
                                } else {
                                    input {
                                        r#type: "email",
                                        value: "{email}",
                                        oninput: move |e: Event<FormData>| email.set(e.value().to_string()),
                                    }
                                }
                            }
                        }

                        tr {
                            td {
                                label { "Username : " }
                            }
                            td {
                                input {
                                    r#type: "text",
                                    value: "{username}",
                                    oninput: move |e: Event<FormData>| username.set(e.value().to_string()),
                                }
                            }
                        }

                        tr {
                            td {
                                label { "Role : " }
                            }
                            td {
                                // Don't allow a user to update their own role
                                if *user_id.read() == auth.id() {
                                    label { "{role}" }
                                } else {
                                    select {
                                        value: "{role}",
                                        onchange: move |e: Event<FormData>| role.set(e.value().to_string()),
                                        option { value: "User", "User" }
                                        option { value: "Admin", "Admin" }
                                    }
                                }
                            }
                        }

                        tr {
                            // Don't allow a user to update their own active status
                            if *user_id.read() == auth.id() {
                                td {
                                    label { "Active : " }
                                }
                                td {
                                    label { "{is_active}" }
                                }
                            } else {
                                td {
                                    label { "" }
                                }
                                td {
                                    input {
                                        r#type: "checkbox",
                                        checked: *is_active.read(),
                                        onchange: move |e: Event<FormData>| is_active.set(e.value() == "true"),
                                    }
                                    " Active"
                                }
                            }
                        }
                    }
                }
            }

            div {
                // Save user details
                button {
                    id: "page-button",
                    onclick: move |_| {
                        spawn(async move {
                            let user = shared::User {
                                id: if EDITING_USER.read().user.is_some() {
                                    *user_id.read()
                                } else {
                                    -1
                                },
                                username: username.read().clone(),
                                email: email.read().clone(),
                                role: match role.read().as_str() {
                                    "Admin" => Role::Admin,
                                    _ => Role::User,
                                },
                                is_active: *is_active.read(),
                                change_password: *change_password.read(),
                            };

                            log::debug!("The user : {:?}", user);
                            if let Some(_) = api::save_user(&user).await {
                                log::debug!("User saved successfully");
                                nav.push(Route::UserListPage {});
                            } else {
                                log::debug!("Failed to save user");
                            }
                        });

                    },
                    "Save"
                }
                button {
                    id: "page-button",
                    onclick: move |_| {
                        nav.push(Route::UserListPage {});
                    },
                    "Cancel"
                }
            }
        }
    }
}