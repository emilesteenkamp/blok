//! The program I/O panel: renders [`RunEvent`] output and, when the
//! program is waiting on [`RunEvent::InputRequested`], an input box that
//! answers it with a [`DebugCommand::ProvideInput`].
//!
//! Depends only on `dioxus` and `runtime-protocol` — never `runtime`
//! directly, per the runtime contract in `docs/technical-design.md`, so
//! this crate has no idea a Lua VM is even involved.

#![deny(missing_docs)]

mod derive;

use dioxus::prelude::*;
use runtime_protocol::{DebugCommand, RunEvent};

use derive::{is_still_running, is_waiting_for_input, output_lines};

const CONSOLE_CSS: Asset = asset!("/assets/console.css");

/// Renders `events`' accumulated output and, when the most recent event is
/// an unanswered [`RunEvent::InputRequested`], an input box — submitting
/// it calls `on_command` with [`DebugCommand::ProvideInput`].
#[component]
pub fn Console(events: ReadSignal<Vec<RunEvent>>, on_command: EventHandler<DebugCommand>) -> Element {
    let mut input_value = use_signal(String::new);

    // Always-tail: a real terminal never makes you scroll to see what just
    // happened, so this follows new output to the bottom as it arrives.
    use_effect(move || {
        let _ = events.read().len();
        spawn(async move {
            let _ = document::eval(
                "const el = document.getElementById('console-output'); \
                 if (el) { el.scrollTop = el.scrollHeight; }",
            )
            .await;
        });
    });

    let events_snapshot = events.read();
    let lines = output_lines(&events_snapshot);
    let waiting = is_waiting_for_input(&events_snapshot);
    let running = is_still_running(&events_snapshot);

    rsx! {
        document::Stylesheet { href: CONSOLE_CSS }
        div { class: "console-panel",
            div { class: "console-output", id: "console-output",
                if lines.is_empty() {
                    p { class: "console-empty",
                        "Console output will appear here once you run your program."
                    }
                } else {
                    for (i, line) in lines.iter().enumerate() {
                        p { key: "{i}", class: "console-line", "{line}" }
                    }
                }
                if running {
                    span { class: "console-cursor" }
                }
            }
            if waiting {
                div { class: "console-input-row",
                    span { class: "console-input-prompt", "❯" }
                    input {
                        class: "console-input",
                        value: "{input_value}",
                        placeholder: "Type your answer and press Enter",
                        autofocus: true,
                        oninput: move |event| input_value.set(event.value()),
                        onkeydown: move |event| {
                            if event.key() == Key::Enter {
                                let value = std::mem::take(&mut *input_value.write());
                                on_command.call(DebugCommand::ProvideInput(value));
                            }
                        },
                    }
                }
            }
        }
    }
}
