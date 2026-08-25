use dioxus::prelude::*;
use crate::{Route, services::api, state::auth::AUTH};

/// Shared components

// Navigation bar
#[component]
pub fn Navbar() -> Element {
    let auth = AUTH.read();

    rsx! {
        div {
            nav { id: "navbar",

                Link { to: Route::HomePage {}, "Home" }

                if auth.user.is_none() || auth.requires_password_change() {
                    Link { to: Route::LoginPage {}, "Login" }
                }

                if let Some(_user) = &auth.user {
                    if !auth.requires_password_change() {
                        button {
                            onclick: move |_| {
                                let nav = navigator();
                                spawn(async move {
                                    let _ = api::logout().await;
                                    AUTH.write().user = None;
                                    nav.push(Route::LogoutPage {});
                                });
                            },
                            "Logout"
                        }
                    }
                }
            }
        }
        Outlet::<Route> {}
    }
}
