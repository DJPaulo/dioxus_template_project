use dioxus::prelude::*;
use crate::state::ui::CONFIRM_MODAL;

#[component]
pub fn ConfirmModal() -> Element {
    let modal = CONFIRM_MODAL.read();

    if !modal.open {
        return rsx! {};
    }

    let message = modal.message.clone();
    let on_yes = modal.on_yes.clone();
    let on_no  = modal.on_no.clone();

    rsx! {
        div { class: "modal-backdrop",
            div { class: "modal",
                p { "{message}" }

                button {
                    id: "page-button",
                    onclick: move |_| {
                        if let Some(cb) = &on_yes {
                            cb();
                        }
                        CONFIRM_MODAL.write().close();
                    },
                    "Yes"
                }
                button {
                    id: "page-button",
                    onclick: move |_| {
                        if let Some(cb) = &on_no {
                            cb();
                        }
                        CONFIRM_MODAL.write().close();
                    },
                    "No"
                }
            }
        }
    }
}
