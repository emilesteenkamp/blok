---
name: verify-ui-change
description: Use after any Dioxus UI/CSS change in blok (app shell, console, debug-panel, block-editor, or any crate's example preview harness) that needs visual confirmation. Covers launching a desktop preview, safely screenshotting just that window, scripting multi-state harnesses, and the Dioxus/WKWebView quirks that commonly cause false "it's broken" readings.
---

# Verifying a Dioxus desktop UI change

Rust/CSS looking correct is not the same as the window looking correct — Dioxus desktop renders through WKWebView (via `wry`), which has a few quirks that don't show up by reading the code. This skill is the checklist for actually looking at the result, and for not misdiagnosing a capture artifact as a rendering bug (that mistake cost a long debugging detour once already).

## Launch and screenshot, safely

1. Kill any stale instance first, so you're not screenshotting an old window: `pkill -9 -f "examples/preview"` or `pkill -9 -f "target/debug/app"` as appropriate.
2. Launch in the background: `cargo run -p <crate> --example preview > /tmp/preview.log 2>&1 &` (or `cargo run -p app ...` for the shell). Give it a couple seconds to build and open.
3. Find its window ID — **never use a full-screen `screencapture`**, it can expose whatever else is on the user's desktop (this happened once already and is a hard rule now):
   ```
   swift .claude/skills/verify-ui-change/list_windows_by_pid.swift <PID>
   ```
   This prints `<CGWindowID>\t<title>` for every on-screen window owned by that PID.
4. **Bring the window to front before capturing.** This is the step that's easy to skip and causes the most confusion:
   ```
   osascript -e 'tell application "System Events" to set frontmost of (first process whose unix id is <PID>) to true'
   ```
   `screencapture -l <windowID>` on a window that is **not frontmost** can silently return a blank or stale-looking image even though the app is rendering correctly — this is a macOS capture artifact, not a bug in your CSS. If a screenshot comes back blank, activate the window and recapture *before* you start suspecting the markup.
5. Capture: `screencapture -l <windowID> -o <path>.png`, then view it with Read.
6. When done, kill the process again rather than leaving it running.

## Driving multi-state preview harnesses

Preview harnesses (e.g. `crates/console/examples/preview.rs`) often expose buttons to switch between mocked states (empty / populated / waiting-for-input, etc.). You can click these from the outside via Accessibility instead of adding temporary debug code:

```
osascript -e '
tell application "System Events"
    set targetProc to first process whose unix id is <PID>
    tell targetProc
        click button "<Button Label>" of window 1
    end tell
end tell'
```

If that fails with "Can't get button... of window 1", the button is nested deeper in WKWebView's accessibility tree (common — it's not a direct child of the window). Dump the real path first:

```
osascript -e '
tell application "System Events"
    set targetProc to first process whose unix id is <PID>
    tell targetProc
        return entire contents of window 1
    end tell
end tell'
```

This prints the full element path (typically something like `button "X" of UI element 1 of scroll area 1 of group 1 of group 1 of window 1`) — use that exact path in the `click button` call.

## Dioxus/WKWebView gotchas to know before writing the CSS/RSX

These are easy to get wrong because they work fine in a regular browser and only misbehave in this specific renderer:

- **`document::eval(...)` must be spawned and awaited, not called bare.** Calling it directly inside a component body or un-awaited inside `use_effect` drops the returned `Eval` immediately and the JS likely never runs. Always:
  ```rust
  use_effect(move || {
      spawn(async move {
          let _ = document::eval("...").await;
      });
  });
  ```
- **`flex: 1; overflow-y: auto` needs `min-height: 0`.** Without it, a flex child never becomes the bounded/scrollable element — it grows to fit its content instead, and nothing scrolls. This is a general flexbox fact, not Dioxus-specific, but it's the first thing to check when an output/scroll panel won't scroll.
- **`height: 100%` does not reliably resolve through a flex-item ancestor chain in Dioxus desktop's WKWebView**, even when the ancestor's actual computed height is correct. A flex child given `height: 100%` can render at its content's size instead of the constrained size. Workaround: skip percentage heights for this case and use absolute-fill positioning instead —
  ```css
  .host-of-the-element { position: relative; } /* on the flex-item ancestor */
  .the-element {
    position: absolute;
    top: 0; right: 0; bottom: 0; left: 0;
  }
  ```
- **Standalone `examples/preview.rs` harnesses don't inherit the app shell's CSS custom properties.** Each crate's CSS is written against `var(--color-bg)`, `var(--font-mono)`, etc., which are normally defined once in `crates/app/assets/theme.css`'s `:root`. A harness that doesn't depend on `app` needs its own mirrored `:root` block (a `THEME_TOKENS` const rendered via `style { "{THEME_TOKENS}" }` at the top of the harness works well) — otherwise every `var(--...)` silently resolves to nothing and the harness renders completely unstyled, which looks like a CSS loading bug but isn't.

## After a fix, before calling it done

Re-run the full checklist above against the actual fix, including process cleanup and removal of any temporary debug code (stray `eprintln!`, DOM-dump `document::eval` calls, debug outline styles, mocked-data seeds left over from testing) — none of that should land in the committed version.
