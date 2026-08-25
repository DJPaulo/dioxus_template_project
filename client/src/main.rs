mod app;
mod state;
mod components;
mod pages;
mod services;

use dioxus::prelude::*;
use crate::components::navigation::Navbar;
use crate::pages::{
    home::HomePage,
    login::LoginPage,
    logout::LogoutPage,
    user::user_list::UserListPage,
    user::user_add_edit::UserAddEditPage,
    user::user_password_change::UserPasswordChangePage,
    user::user_password_reset::UserPasswordResetPage,
    user::user_forgot_password::UserForgotPasswordPage,
};

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");


#[derive(Debug, Clone, Routable, PartialEq)]
enum Route {
    #[layout(Navbar)]

    #[route("/")]
    HomePage {},

    #[route("/login")]
    LoginPage {},

    #[route("/logout")]
    LogoutPage {},

    #[route("/user")]
    UserListPage {},

    #[route("/user/add_edit")]
    UserAddEditPage {},

    #[route("/user/change_password")]
    UserPasswordChangePage {},

    #[route("/user/reset_password/:token")]
    UserPasswordResetPage { token: String },

    #[route("/user/forgot_password")]
    UserForgotPasswordPage {},

//    #[route("/dashboard")]
//    Dashboard {},

//    #[route("/admin")]
//    AdminPanel {},

//    #[route("/admin/users")]
//    UserManagement {},
}


fn main() {
    console_log::init_with_level(log::Level::Debug).unwrap();
    console_error_panic_hook::set_once();

    dioxus::launch(app::App);
}

