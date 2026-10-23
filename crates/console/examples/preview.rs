//! Standalone visual preview of the `Console` component, fed mocked
//! `RunEvent` streams — no `runtime` crate, no app shell. Buttons switch
//! between the states a real run would produce, so all of them can be
//! reviewed without wiring this up to anything real yet.
//!
//! Run with: `cargo run -p console --example preview`

use console::Console;
use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;
use runtime_protocol::{DebugCommand, RunEvent};

/// Mirrors `crates/app/assets/theme.css`'s `:root` tokens — `console.css`
/// (like every crate's CSS) is written against these custom properties,
/// normally defined once by the app shell. This standalone preview has no
/// app shell, so it defines them itself to render the real thing rather
/// than a page where every `var(--undefined)` silently falls back to
/// nothing.
const THEME_TOKENS: &str = r#"
:root {
  --color-bg: #0b0d10;
  --color-fg: #fafafa;
  --color-fg-muted: #9ca3af;
  --color-border: #2a2d33;
  --color-accent: #4f6bff;
  --font-sans: "Inter", "IBM Plex Sans", -apple-system, BlinkMacSystemFont, sans-serif;
  --font-mono: "JetBrains Mono", "Fira Code", ui-monospace, monospace;
  --radius-sm: 4px;
  --radius-md: 8px;
  --space-1: 4px;
  --space-2: 8px;
  --space-3: 16px;
  --space-4: 24px;
}
body {
  margin: 0;
}
"#;

fn main() {
    dioxus::LaunchBuilder::desktop()
        .with_cfg(Config::new().with_window(WindowBuilder::new().with_title("console preview")))
        .launch(Preview);
}

#[component]
fn Preview() -> Element {
    let mut events = use_signal(Vec::<RunEvent>::new);
    let mut last_command = use_signal(|| None::<DebugCommand>);

    rsx! {
        style { "{THEME_TOKENS}" }
        div {
            style: "display: flex; flex-direction: column; height: 100vh; background: var(--color-bg); color: var(--color-fg); font-family: var(--font-sans);",
            div { style: "display: flex; gap: 8px; padding: 8px; border-bottom: 1px solid var(--color-border);",
                button { onclick: move |_| events.set(vec![]), "Empty" }
                button {
                    onclick: move |_| {
                        events.set(vec![
                            RunEvent::Output("counting from 1 to 3".to_string()),
                            RunEvent::Output("1".to_string()),
                            RunEvent::Output("2".to_string()),
                            RunEvent::Output("3".to_string()),
                            RunEvent::Finished,
                        ])
                    },
                    "Some output"
                }
                button {
                    onclick: move |_| {
                        events.set(vec![
                            RunEvent::Output("What's your name?".to_string()),
                            RunEvent::InputRequested,
                        ])
                    },
                    "Waiting for input"
                }
            }
            div { style: "flex: 1; min-height: 0; position: relative;",
                Console {
                    events,
                    on_command: move |command| last_command.set(Some(command)),
                }
            }
            div { style: "padding: 8px; border-top: 1px solid var(--color-border); font-family: var(--font-mono); font-size: 0.8rem; color: var(--color-fg-muted);",
                "last command sent: {last_command:?}"
            }
        }
    }
}
