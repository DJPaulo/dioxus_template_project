use std::rc::Rc;
use dioxus::prelude::*;
use shared::User;
use crate::{Route, services::api, };
use crate::state::{auth::AUTH, user_edit::EDITING_USER, ui::CONFIRM_MODAL};
 
#[component]
pub fn UserListPage() -> Element {
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

    let users = use_signal(|| Vec::<User>::new());
    let mut loading = use_signal(|| false);
    let error = use_signal(|| String::new());
    let mut has_loaded = use_signal(|| false);

    if !*has_loaded.read() {
        has_loaded.set(true);
        loading.set(true);

        spawn({
            let mut users = users.clone();
            let mut loading = loading.clone();
            let mut error = error.clone();

            async move {
                match api::list_users().await {
                    Ok(fetched_users) => users.set(fetched_users),
                    Err(message) => error.set(message),
                }

                loading.set(false);
            }
        });
    }

    let error_message = error.read().clone();

    rsx! {
        div { id: "admin-user-maintenance",
            h1 { "User Maintenance" }

            if *loading.read() {
                p { "Loading users..." }
            } else if !error_message.is_empty() {
                p { class: "text-red-600", "{error_message}" }
            } else {
                button {
                    id: "add-new-button",
                    onclick: {
                        move |_| {
                            EDITING_USER.write().user = None;
                            nav.push(Route::UserAddEditPage {});
                        }
                    },
                    "+ New User"
                }
                table {
                    thead {
                        tr {
                            th { style: "display: none;", "ID" }
                            th { "Username" }
                            th { "Email" }
                            th { "Role" }
                            th { "Active" }
                            th { "" }
                        }
                    }
                    tbody {
                        for user in users.read().iter() {
                            tr {
                                td { style: "display: none;", "{user.id}" }
                                td { "{user.username}" }
                                td { "{user.email}" }
                                td { "{user.role:?}" }
                                td {
                                    if user.is_active {
                                        "Yes"
                                    } else {
                                        "No"
                                    }
                                }
                                td {
                                    button {
                                        id: "table-button",
                                        onclick: {
                                            let user = user.clone();
                                            move |_| {
                                                EDITING_USER.write().user = Some(user.clone());
                                                nav.push(Route::UserAddEditPage {});
                                            }
                                        },
                                        "Edit"
                                    }
                                    // User admins can't delete themselves or reset their own passwords
                                    if user.id != auth.id() {
                                        button {
                                            id: "table-button",
                                            onclick: {
                                                let user = user.clone();

                                                move |_| {
                                                    EDITING_USER.write().user = Some(user.clone());

                                                    let user_for_cb = user.clone();

                                                    CONFIRM_MODAL
                                                        .write()
                                                        .open(
                                                            format!("Are you sure you want to delete user {}?", user.email),
                                                            Some(
                                                                Rc::new(move || {
                                                                    let user_id = user_for_cb.id;
                                                                    let mut users = users.clone();
                                                                    let mut loading = loading.clone();
                                                                    let mut error = error.clone();
                                                                    spawn(async move {
                                                                        if api::delete_user(user_id).await.is_some() {
                                                                            log::debug!("User deleted successfully");
                                                                            loading.set(true);

                                                                            match api::list_users().await {
                                                                                Ok(fetched_users) => users.set(fetched_users),
                                                                                Err(message) => error.set(message),
                                                                            }

                                                                            loading.set(false);
                                                                            //nav.push(Route::AdminUserListPage {});
                                                                        } else {
                                                                            log::debug!("Failed to delete user");
                                                                        }
                                                                    });
                                                                }),
                                                            ),
                                                            Some(
                                                                Rc::new(move || {
                                                                    spawn(async move {
                                                                        nav.push(Route::UserListPage {});
                                                                    });
                                                                }),
                                                            ),
                                                        );
                                                }
                                            },
                                            "Delete"
                                        }

                                        button {
                                            id: "table-button",
                                            onclick: {
                                                let user = user.clone();

                                                move |_| {
                                                    EDITING_USER.write().user = Some(user.clone());

                                                    let user_for_cb = user.clone();

                                                    CONFIRM_MODAL
                                                        .write()
                                                        .open(
                                                            format!(
                                                                "Are you sure you want to reset the password for user {}?",
                                                                user.email,
                                                            ),
                                                            Some(
                                                                Rc::new(move || {
                                                                    let user = user_for_cb.clone();
                                                                    let mut users = users.clone();
                                                                    let mut loading = loading.clone();
                                                                    let mut error = error.clone(); // nav.push(Route::AdminUserListPage {});
                                                                    spawn(async move {
                                                                        if api::admin_request_user_password_reset(user)
                                                                            .await
                                                                            .is_ok()
                                                                        {
                                                                            log::debug!("User password reset successfully");
                                                                            loading.set(true);
                                                                            match api::list_users().await {
                                                                                Ok(fetched_users) => users.set(fetched_users),
                                                                                Err(message) => error.set(message),
                                                                            }
                                                                            loading.set(false);
                                                                        } else {
                                                                            log::debug!("Failed to reset user password");
                                                                        }
                                                                    });
                                                                }),
                                                            ),
                                                            Some(
                                                                Rc::new(move || {
                                                                    spawn(async move {
                                                                        nav.push(Route::UserListPage {});
                                                                    });
                                                                }),
                                                            ),
                                                        );
                                                }
                                            },
                                            "Reset"
                                        }
                                    }
                                }
                            
                            }
                        }
                    }
                }
            }
        }
    }
}