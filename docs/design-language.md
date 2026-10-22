# blok — Design Language

Oct 22, 2026 · @Emile

**Theme: “Quiet Capability”** — as precise as an IDE, as approachable as a sketchpad. Blocks should read as tools, not toys: modern and clean, not a kids' aesthetic.

## Core visual identity

| Element | Recommendation | Notes |
| --- | --- | --- |
| Color palette | Neutral base, first-class dark mode (near-black `#0B0D10` dark, near-white `#FAFAFA` light); one restrained accent for primary actions and focus (e.g. indigo `#4F6BFF`); block categories get muted, desaturated hues at varied lightness rather than a full bright rainbow | Color should code meaning (category, state), not decorate. Nothing candy-bright or neon. |
| Typography | Primary UI: a clean grotesk/humanist sans (Inter, IBM Plex Sans); monospace for variable values, console output, and generated Lua (JetBrains Mono, Fira Code) | Monospace for code-like content signals "this is a real programming tool," not a toy. Avoid bubbly display fonts (Poppins, Rubik) — they read as playful/kids'. |
| Shapes & layout | Modest corner radius (4–8px); flat blocks with crisp 1px borders, not heavy drop shadows; elevation reserved for overlays/menus only; generous whitespace on the workspace canvas, denser panels for the variable inspector and console; consistent 4/8px spacing scale | Scratch's heavy rounding + puzzle-notch blocks read as toys — blok's blocks should read as clean, flat components. |
| Iconography | Thin-to-medium line icons, consistent stroke weight (Lucide-style) | No filled or cartoon icons. |

## Interaction design

| Principle | Implementation |
| --- | --- |
| Clarity over decoration | Every visual signal maps to real program state — the active block, a changed variable — never decoration without meaning |
| Snap, don't float | Drag-and-drop gives immediate, precise snap feedback (magnetism + outline highlight), not bounce or sound effects |
| Legible at a glance | The active execution block and changed variables get a visible but restrained highlight (color + weight change), not an animation-heavy treatment |
| Progressive disclosure | The console and variable inspector stay out of the way until a program runs, then surface exactly what changed |

## Motion & transitions

- Fast, purposeful easing (120–180ms, ease-out) — never bouncy or elastic
- Step execution highlights the active block with a brief, restrained pulse; no full-screen transitions
- Panel and workspace transitions are a simple fade or slide — utilitarian, closer to Linear or Zed than an iOS app switcher
- Motion always communicates a state change; it is never decoration for its own sake

## Tone & copy

| Type | Example |
| --- | --- |
| Voice | Clear, precise, encouraging — like a good TA, not a toy and not a terminal |
| Example microcopy | "Step 4 of 9 — `count` is now 3." / "Drag a Start block in to run your program." |
| Empty states | Calm and instructive — "Nothing to run yet. Start by dragging in a block." instead of cutesy filler or a bare "No program found." |

## Brand personality keywords

| Keyword | Interpretation |
| --- | --- |
| Precise | Every interaction and visual reflects exactly what the program is doing |
| Approachable | Low-barrier for beginners, without babying them |
| Capable | Feels like a real tool you could grow into, not a toy you'll outgrow |
| Calm | Restrained color and motion keep focus on the code, not the chrome |
| Honest | The UI never hides or decorates over the program's actual state |

## Design language inspiration

| Source | What to borrow |
| --- | --- |
| Linear | Crisp utility, restrained color, perfect spacing |
| Zed / VS Code | Dense, legible dev-tool chrome; respect for monospace and code |
| Figma | Precise direct-manipulation canvas, clean selection and snap feedback |
| Apple HIG | Clarity and motion hierarchy — not its warmth or playfulness |
| Explicitly avoid | Scratch/Duolingo-style bright primary colors, puffy rounded shapes, mascots, sound effects |

## Summary tagline

*"Quiet capability: a design language that looks like a tool, not a toy — simple enough to start, serious enough to stay."*
