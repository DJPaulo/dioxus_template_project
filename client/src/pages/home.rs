use dioxus::prelude::*;
use crate::{Route, state::auth::AUTH};
//use crate::pages::admin::admin_user_list::AdminUserListPage;
use crate::HEADER_SVG;


#[component]
pub fn HomePage() -> Element { 
    let auth = AUTH.read();
    let nav = navigator();


    if auth.requires_password_change() {
        nav.replace(Route::UserPasswordChangePage {});
        return rsx! { "Password change required" };
    }

    rsx! {
        if auth.user.is_none() {
            div { id: "hero",
                img { src: HEADER_SVG, id: "header" }
                div { id: "links",
                    a { href: "https://dioxuslabs.com/learn/0.7/", "📚 Learn Dioxus" }
                    a { href: "https://dioxuslabs.com/awesome", "🚀 Awesome Dioxus" }
                    a { href: "https://github.com/dioxus-community/", "📡 Community Libraries" }
                    a { href: "https://github.com/DioxusLabs/sdk", "⚙️ Dioxus Development Kit" }
                    a { href: "https://marketplace.visualstudio.com/items?itemName=DioxusLabs.dioxus",
                        "💫 VSCode Extension"
                    }
                    a { href: "https://discord.gg/XgGxMSkvUM", "👋 Community Discord" }
                }
            }
        }

        if let Some(user) = &auth.user {
            div { id: "hero",
                h1 { "Hi {user.username}, please select from the following options:" }
                div { id: "links",
                    Link { to: Route::HomePage {}, " Option 1" }
                    Link { to: Route::HomePage {}, " Option 2" }
                    Link { to: Route::HomePage {}, " Option 3" }

                    //Admin only links
                    if auth.is_admin() {
                        Link { to: Route::UserListPage {}, "Admin - User Maintenance" }
                    }
                }
            }
        }
    }  
}