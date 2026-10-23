use dioxus::prelude::*;

const THEME_CSS: Asset = asset!("/assets/theme.css");

fn main() {
    #[cfg(feature = "desktop")]
    {
        use dioxus::desktop::{Config, WindowBuilder};

        dioxus::LaunchBuilder::desktop()
            .with_cfg(Config::new().with_window(WindowBuilder::new().with_title("blok")))
            .launch(App);
    }

    #[cfg(not(feature = "desktop"))]
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: THEME_CSS }
        div { class: "app-shell",
            p { class: "empty-state",
                span { class: "empty-state-prompt", "❯" }
                span { "Nothing to run yet. Start by dragging in a block." }
                span { class: "cursor" }
            }
        }
    }
}
