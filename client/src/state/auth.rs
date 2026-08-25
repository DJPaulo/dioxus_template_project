use dioxus::prelude::*;
use shared::User;

pub static AUTH: GlobalSignal<AuthState> = GlobalSignal::new(|| AuthState { user: None });

#[derive(Clone, Default)]
pub struct AuthState {
    pub user: Option<User>,
}

impl AuthState {
    pub async fn load_session(&mut self) {
        if let Some(user) = crate::services::api::me().await {
            self.user = Some(user);
        } else {
            self.user = None;
        }
    }

    pub fn is_admin(&self) -> bool {
        matches!(self.user.as_ref().map(|u| &u.role), Some(shared::Role::Admin))
    }

    pub fn requires_password_change(&self) -> bool {
        matches!(self.user.as_ref().map(|u| &u.change_password), Some(true))
    }

    pub fn id(&self) -> i32 {
        match self.user.as_ref().map(|u| u.id) {
            Some(id) => id,
            None => 0,
        }
    }
}
