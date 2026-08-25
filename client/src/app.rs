use dioxus::prelude::*;
use dioxus_router::Router;
use crate::{FAVICON, MAIN_CSS, TAILWIND_CSS, Route};
use crate::components::confirm_modal::ConfirmModal;
use crate::state::auth::AUTH;

#[component]
pub fn App() -> Element {
    
    use_future(|| async {
        AUTH.write().load_session().await;
    });

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        ConfirmModal {}
        Router::<Route> {}
    }
}
