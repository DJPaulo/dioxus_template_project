use dioxus::prelude::*;
use shared::User;

pub static EDITING_USER: GlobalSignal<EditingUserState> = GlobalSignal::new(|| EditingUserState { user: None });

#[derive(Clone, Default)]
pub struct EditingUserState {
    pub user: Option<User>,
}