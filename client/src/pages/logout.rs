use dioxus::prelude::*;
use crate::Route;
use crate::state::auth::AUTH;


#[component]
pub fn LogoutPage() -> Element {
    let auth = AUTH.read();
    let nav = navigator();

    if auth.user.is_none() {
        nav.replace(Route::LoginPage {});
        return rsx! { "Forbidden" };
    }

    rsx! {
        div { id: "logout",
            h1 { "You have been logged out" }
        }
    }
}