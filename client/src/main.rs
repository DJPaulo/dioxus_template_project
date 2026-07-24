mod app;

use app::App as AppComponent;

fn main() {
    dioxus::launch(AppComponent);
}

