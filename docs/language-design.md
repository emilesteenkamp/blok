# blok — Language Design

Oct 22, 2026 · @Emile

> Live, editable version: https://claude.ai/artifact/JVHeA6Ma1oFoKXhZhwJLJY (this file is a text snapshot)

## Overview

blok's "language" is the set of blocks a program can be built from and what each one means — independent of how those blocks are drawn or dragged (`docs/design-language.md`) and independent of how the resulting tree is executed (`docs/technical-design.md`'s Execution runtime section). This doc is the reference for that vocabulary: implemented in the `block-ir` crate, consumed by `block-editor` (to render and constrain what snaps into what) and `lua-transpiler` (to lower a tree to Lua).

**MVP scope** (per `README.md`): statements, variables, expressions, conditionals, loops, and built-in data structures. No user-defined functions or structs — those are post-MVP.

## Core language vs. standard library

**Decision**: the language is split into two deliberately different kinds of block.

- **Core language** — control flow, operators, variables, and the two built-in data structures. Each gets its own dedicated variant in `block_ir::ExpressionKind` or `block_ir::StatementKind`. This set is closed: adding to it means changing the AST shape, which `block-editor` and `lua-transpiler` both depend on.
- **Standard library (packages)** — math, strings, and future packages. These are *not* core language constructs and never get their own AST variant. Every stdlib call is represented by the single `ExpressionKind::PackageCall { package, function, args }` node; a function's name, arity, and argument/return types are plain data in a `Package`/`FunctionSignature` registry (`block_ir::packages`). Adding a stdlib function means adding a row to that registry, not touching the AST.

**Rationale**: keywords like `if`/`switch` and data structures like `Map` are part of what makes blok a general-purpose language, so they deserve first-class syntax. Math and string helpers are conveniences on top of that language, not part of it — modeling them as an open-ended, data-driven registry keeps the core AST small and stable while leaving room to grow the stdlib (or, post-MVP, let a document opt into only the packages it needs) without ever changing `block-ir`'s shape.

## Core language reference

### Variables

- `VariableDecl { name, value }` — declares a new variable with an initial value.
- `Assign { name, value }` — assigns a new value to an existing variable.

### Control flow

- `If { condition, then_body, else_body }` — runs `then_body` or `else_body` depending on `condition` (must be `Bool`).
- `Switch { subject, cases, default }` — matches `subject` against each case's value in order, running the first match's body or `default`. Case values must be literals (checked by `validate`) — they're compile-time constants, not arbitrary expressions.
- `WhileLoop { condition, body }` — repeats `body` while `condition` (`Bool`) holds, re-checked before every iteration.
- `ForLoop { variable, start, end, step, body }` — repeats `body` with `variable` counting from `start` to `end` (`Number`), advancing by `step` each time (default `1`). A literal `step` of `0` is rejected by `validate` — it would loop forever.

### Expressions & operators

- Literals: `NumberLiteral`, `StringLiteral`, `BoolLiteral`.
- `Variable(name)` — a reference to a variable. The MVP has no symbol table, so a variable's `ValueKind` is always `Any` (unknown until runtime) rather than inferred from its declaration.
- `BinaryOp { op, left, right }` — arithmetic (`Add`/`Sub`/`Mul`/`Div`/`Mod`, operands and result `Number`), comparison (`Lt`/`LtEq`/`Gt`/`GtEq`, `Number` operands, `Bool` result; `Eq`/`NotEq`, any operand kind, `Bool` result), logical (`And`/`Or`, `Bool` operands and result).
- `UnaryOp { op, operand }` — `Negate` (`Number`), `Not` (`Bool`).

### Built-in data structures

Two native data structures — no user-defined structs in the MVP (post-MVP, per `README.md`):

- **List** — `ListLiteral`, `ListGet { list, index }`, `ListLength { list }` (expressions), and `ListAppend`/`ListSet` (statements, mutate in place). Lists are **0-indexed** — `ListGet { index: 0 }` is the first element — matching Python/JS and general-purpose-language expectations rather than Lua's native 1-indexing. `lua-transpiler` converts at the codegen boundary (`list[index + 1]`), so this is purely a block-ir/editor-facing convention; nothing about it leaks into the generated Lua's own semantics.
- **Map** — `MapLiteral` (a vector of key/value expression pairs — keys aren't restricted to a particular `ValueKind` in the MVP), `MapGet { map, key }`, `MapLength { map }` (expressions), and `MapSet` (statement, mutates in place). `MapLength` can't be a single Lua expression (counting non-array table entries needs a `pairs()` loop), so `lua-transpiler` emits a small polyfill function for it — see `docs/technical-design.md`'s Execution runtime section.

### Console I/O

- `Print { value }` (statement) — writes `value` (any `ValueKind`) to the console.
- `ReadInput` (expression) — reads one line of console input, always a `String`.

## The type system

Every expression has a `ValueKind`: `Number`, `String`, `Bool`, `List`, `Map`, or `Any`. `Any` is a bidirectional wildcard:

- As the *expected* kind of a socket, it means "accepts anything" (e.g. `print`'s argument).
- As the *found* kind of an expression, it means "not statically known" (e.g. a `Variable` reference, a `ListGet`/`MapGet` result) — accepted optimistically rather than rejected, the same way the generated Lua would only fail at runtime, not at validation time.

`ValueKind::satisfies(self, expected)` is the single compatibility check both `block-editor` (deciding what a block can snap into, live, while the user drags) and `block_ir::validate` (checking a whole tree at once) are built on — the nesting/connection rule data stays in one place rather than being re-derived by each.

`validate(&Program)` walks a tree and reports every violation it finds (not just the first): duplicate `BlockId`s, `ValueKind` mismatches, non-literal `switch` case values, a literal-`0` `for`-loop step, and unknown packages/functions or wrong argument counts/types in a `PackageCall`.

## Standard library packages

Packages are listed in `block_ir::packages`. Only functions piccolo implements natively are included — see `docs/technical-design.md`'s "known gaps" row; piccolo has no string pattern matching (`find`/`match`/`gsub`/`format`), so none of that is exposed as a package function.

| Package | Functions |
| --- | --- |
| `math` | `abs`, `sqrt`, `floor`, `ceil`, `min`, `max`, `random` — all native to piccolo's `math` library |
| `strings` | `length`, `upper`, `lower`, `reverse`, `concat` — native to piccolo's `string` library (pattern matching excluded) |

Adding a package function: add a `FunctionSignature` row to the relevant `Package` constant (or a new `Package` for a new package) in `block_ir::packages` — no AST change, no `block-editor`/`lua-transpiler` changes beyond what's needed to expose it in the palette and lower the call to Lua.

## Block categories

Every block has a `Category` (`Variable`, `ControlFlow`, `Expression`, `DataStructure`, `Io`, `Package`), used by `block-editor` to assign the muted, desaturated category colors from `docs/design-language.md` — kept here as data so the editor never has to re-derive or duplicate the mapping.

## Deferred (post-MVP)

Per `README.md`: user-defined functions, user-defined data structures (structs), canvas I/O. When these land, expect: a new core `StatementKind` for function definitions/calls (a core-language construct, not a package — defining new vocabulary isn't the same as calling an existing stdlib function), and a similar native-vs-registry judgment call for whatever a "struct" ends up meaning in blok's type system.
