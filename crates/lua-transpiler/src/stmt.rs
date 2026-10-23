//! Lowers [`block_ir::Statement`]s to Lua source, recording a
//! [`crate::SourceMap`] entry for each one and emitting a `__blok_mark`
//! call before each one's own code — see [`crate::lib`]'s module docs for
//! why: piccolo exposes no way to inspect a paused frame's position or
//! locals from outside a callback, so the generated Lua reports both
//! itself, at every statement.

use block_ir::{Statement, StatementKind};

use crate::error::TranspileError;
use crate::expr;
use crate::lua::{escape_string, is_valid_identifier};
use crate::source_map::SourceMap;
use crate::writer::Writer;

/// Lowers a chain of statements (a program body or a nested block),
/// writing each at the writer's current indentation.
///
/// `scope` is the variable-name stack currently in scope at this point in
/// the tree — grown by `VariableDecl`/loop variables and restored by the
/// caller when a nested body like this one finishes, so names declared
/// inside don't leak to code after it (matching Lua's own block scoping).
pub(crate) fn lower_body(
    body: &[Statement],
    w: &mut Writer,
    map: &mut SourceMap,
    scope: &mut Vec<String>,
) -> Result<(), TranspileError> {
    for stmt in body {
        lower(stmt, w, map, scope)?;
    }
    Ok(())
}

fn lower(
    stmt: &Statement,
    w: &mut Writer,
    map: &mut SourceMap,
    scope: &mut Vec<String>,
) -> Result<(), TranspileError> {
    map.record(stmt.id, w.current_line());
    w.writeln(&mark_call(stmt, scope));
    match &stmt.kind {
        StatementKind::VariableDecl { name, value } => {
            let name = checked_identifier(stmt, name)?;
            let value = expr::lower(value)?;
            w.writeln(&format!("local {name} = {value}"));
            scope.push(name);
        }
        StatementKind::Assign { name, value } => {
            let name = checked_identifier(stmt, name)?;
            let value = expr::lower(value)?;
            w.writeln(&format!("{name} = {value}"));
        }
        StatementKind::If {
            condition,
            then_body,
            else_body,
        } => {
            let condition = expr::lower(condition)?;
            w.writeln(&format!("if {condition} then"));
            lower_nested_body(then_body, w, map, scope)?;
            if !else_body.is_empty() {
                w.writeln("else");
                lower_nested_body(else_body, w, map, scope)?;
            }
            w.writeln("end");
        }
        StatementKind::Switch {
            subject,
            cases,
            default,
        } => lower_switch(stmt, subject, cases, default, w, map, scope)?,
        StatementKind::WhileLoop { condition, body } => {
            let condition = expr::lower(condition)?;
            w.writeln(&format!("while {condition} do"));
            lower_nested_body(body, w, map, scope)?;
            w.writeln("end");
        }
        StatementKind::ForLoop {
            variable,
            start,
            end,
            step,
            body,
        } => {
            let variable = checked_identifier(stmt, variable)?;
            let start = expr::lower(start)?;
            let end = expr::lower(end)?;
            let header = match step {
                Some(step) => format!("for {variable} = {start}, {end}, {} do", expr::lower(step)?),
                None => format!("for {variable} = {start}, {end} do"),
            };
            w.writeln(&header);
            let mark = scope.len();
            scope.push(variable);
            w.indented(|w| lower_body(body, w, map, scope))?;
            scope.truncate(mark);
            w.writeln("end");
        }
        StatementKind::Print { value } => {
            w.writeln(&format!("print({})", expr::lower(value)?));
        }
        StatementKind::ListAppend { list, value } => {
            let list = expr::lower(list)?;
            let value = expr::lower(value)?;
            w.writeln(&format!("({list})[#({list}) + 1] = {value}"));
        }
        StatementKind::ListSet { list, index, value } => {
            let list = expr::lower(list)?;
            let index = expr::lower(index)?;
            let value = expr::lower(value)?;
            w.writeln(&format!("({list})[({index}) + 1] = {value}"));
        }
        StatementKind::MapSet { map: m, key, value } => {
            let m = expr::lower(m)?;
            let key = expr::lower(key)?;
            let value = expr::lower(value)?;
            w.writeln(&format!("({m})[{key}] = {value}"));
        }
    }
    Ok(())
}

/// Lowers a nested body (an `if`/`while` branch) at one deeper indent,
/// restoring `scope` to its pre-call length afterward — names declared
/// inside a branch don't leak to code after the branch, matching Lua's own
/// block scoping of `local`.
fn lower_nested_body(
    body: &[Statement],
    w: &mut Writer,
    map: &mut SourceMap,
    scope: &mut Vec<String>,
) -> Result<(), TranspileError> {
    let mark = scope.len();
    w.indented(|w| lower_body(body, w, map, scope))?;
    scope.truncate(mark);
    Ok(())
}

/// The `__blok_mark(id, name, value, ...)` call emitted before a
/// statement's own code: reports the statement's `BlockId` and every
/// in-scope variable's current value, since piccolo gives `runtime` no
/// other way to learn either once paused.
///
/// Ends with `;` — without it, a following statement that happens to
/// start with `(` (`ListAppend`/`ListSet`/`MapSet` all do:
/// `(list)[...] = value`) would parse as a continuation of this call
/// (`__blok_mark(...)(list)[...]`, i.e. calling the mark's — nil — return
/// value) rather than as its own statement. Lua's classic "ambiguous
/// syntax" gotcha; `;` is the standard disambiguator.
fn mark_call(stmt: &Statement, scope: &[String]) -> String {
    let mut args = vec![stmt.id.raw().to_string()];
    for name in scope {
        args.push(escape_string(name));
        args.push(name.clone());
    }
    format!("__blok_mark({});", args.join(", "))
}

/// Lua has no `switch`. Lowered to an if/elseif chain comparing a
/// once-evaluated temporary against each (literal) case value.
fn lower_switch(
    stmt: &Statement,
    subject: &block_ir::Expression,
    cases: &[block_ir::SwitchCase],
    default: &[Statement],
    w: &mut Writer,
    map: &mut SourceMap,
    scope: &mut Vec<String>,
) -> Result<(), TranspileError> {
    let temp = format!("__switch_{}", stmt.id.raw());
    let subject = expr::lower(subject)?;
    w.writeln(&format!("local {temp} = {subject}"));

    for (i, case) in cases.iter().enumerate() {
        let value = expr::lower(&case.value)?;
        let keyword = if i == 0 { "if" } else { "elseif" };
        w.writeln(&format!("{keyword} {temp} == {value} then"));
        lower_nested_body(&case.body, w, map, scope)?;
    }

    if !default.is_empty() {
        if cases.is_empty() {
            w.writeln("if true then");
        } else {
            w.writeln("else");
        }
        lower_nested_body(default, w, map, scope)?;
    }

    if !cases.is_empty() || !default.is_empty() {
        w.writeln("end");
    }
    Ok(())
}

fn checked_identifier(stmt: &Statement, name: &str) -> Result<String, TranspileError> {
    if is_valid_identifier(name) {
        Ok(name.to_string())
    } else {
        Err(TranspileError::InvalidIdentifier {
            block: stmt.id,
            name: name.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use block_ir::{BlockId, Expression, ExpressionKind, SwitchCase};

    fn id(raw: u64) -> BlockId {
        BlockId::from_raw(raw)
    }

    fn num(raw: u64, n: f64) -> Expression {
        Expression {
            id: id(raw),
            kind: ExpressionKind::NumberLiteral(n),
        }
    }

    fn run(body: &[Statement]) -> (String, SourceMap) {
        let mut w = Writer::new();
        let mut map = SourceMap::new();
        let mut scope = Vec::new();
        lower_body(body, &mut w, &mut map, &mut scope).unwrap();
        (w.finish(), map)
    }

    #[test]
    fn variable_decl_emits_mark_then_local() {
        let (src, map) = run(&[Statement {
            id: id(0),
            kind: StatementKind::VariableDecl {
                name: "total".to_string(),
                value: num(1, 0.0),
            },
        }]);
        assert_eq!(src, "__blok_mark(0);\nlocal total = 0\n");
        assert_eq!(map.line_for(id(0)), Some(1));
    }

    #[test]
    fn later_statement_sees_earlier_declared_variable_in_its_mark() {
        let (src, _) = run(&[
            Statement {
                id: id(0),
                kind: StatementKind::VariableDecl {
                    name: "total".to_string(),
                    value: num(1, 0.0),
                },
            },
            Statement {
                id: id(2),
                kind: StatementKind::Print {
                    value: num(3, 1.0),
                },
            },
        ]);
        assert!(src.contains("__blok_mark(2, \"total\", total)"));
    }

    #[test]
    fn variable_declared_inside_if_branch_is_out_of_scope_after_it() {
        let (src, _) = run(&[
            Statement {
                id: id(0),
                kind: StatementKind::If {
                    condition: Expression {
                        id: id(1),
                        kind: ExpressionKind::BoolLiteral(true),
                    },
                    then_body: vec![Statement {
                        id: id(2),
                        kind: StatementKind::VariableDecl {
                            name: "inner".to_string(),
                            value: num(3, 1.0),
                        },
                    }],
                    else_body: vec![],
                },
            },
            Statement {
                id: id(4),
                kind: StatementKind::Print { value: num(5, 1.0) },
            },
        ]);
        // The mark for block 4 (after the if) must not list `inner`.
        assert!(src.contains("__blok_mark(4)"));
        assert!(!src.contains("__blok_mark(4, \"inner\""));
    }

    #[test]
    fn if_without_else_omits_else_clause() {
        let (src, _) = run(&[Statement {
            id: id(0),
            kind: StatementKind::If {
                condition: Expression {
                    id: id(1),
                    kind: ExpressionKind::BoolLiteral(true),
                },
                then_body: vec![Statement {
                    id: id(2),
                    kind: StatementKind::Print { value: num(3, 1.0) },
                }],
                else_body: vec![],
            },
        }]);
        assert_eq!(
            src,
            "__blok_mark(0);\nif true then\n    __blok_mark(2);\n    print(1)\nend\n"
        );
    }

    #[test]
    fn switch_lowers_to_if_elseif_chain_over_a_temp() {
        let (src, _) = run(&[Statement {
            id: id(0),
            kind: StatementKind::Switch {
                subject: Expression {
                    id: id(1),
                    kind: ExpressionKind::Variable("day".to_string()),
                },
                cases: vec![SwitchCase {
                    value: num(2, 1.0),
                    body: vec![Statement {
                        id: id(3),
                        kind: StatementKind::Print { value: num(4, 1.0) },
                    }],
                }],
                default: vec![Statement {
                    id: id(5),
                    kind: StatementKind::Print { value: num(6, 0.0) },
                }],
            },
        }]);
        assert!(src.starts_with("__blok_mark(0);\nlocal __switch_0 = day\n"));
        assert!(src.contains("if __switch_0 == 1 then"));
        assert!(src.contains("else"));
    }

    #[test]
    fn for_loop_without_step_omits_it() {
        let (src, _) = run(&[Statement {
            id: id(0),
            kind: StatementKind::ForLoop {
                variable: "i".to_string(),
                start: num(1, 1.0),
                end: num(2, 5.0),
                step: None,
                body: vec![],
            },
        }]);
        assert_eq!(src, "__blok_mark(0);\nfor i = 1, 5 do\nend\n");
    }

    #[test]
    fn for_loop_variable_is_in_scope_inside_the_body() {
        let (src, _) = run(&[Statement {
            id: id(0),
            kind: StatementKind::ForLoop {
                variable: "i".to_string(),
                start: num(1, 1.0),
                end: num(2, 5.0),
                step: None,
                body: vec![Statement {
                    id: id(3),
                    kind: StatementKind::Print {
                        value: num(4, 1.0),
                    },
                }],
            },
        }]);
        assert!(src.contains("__blok_mark(3, \"i\", i)"));
    }

    #[test]
    fn list_append_indexes_length_plus_one() {
        let (src, _) = run(&[Statement {
            id: id(0),
            kind: StatementKind::ListAppend {
                list: Expression {
                    id: id(1),
                    kind: ExpressionKind::Variable("xs".to_string()),
                },
                value: num(2, 9.0),
            },
        }]);
        assert!(src.contains("(xs)[#(xs) + 1] = 9\n"));
    }

    #[test]
    fn invalid_loop_variable_name_is_an_error() {
        let mut w = Writer::new();
        let mut map = SourceMap::new();
        let mut scope = Vec::new();
        let body = [Statement {
            id: id(0),
            kind: StatementKind::ForLoop {
                variable: "end".to_string(),
                start: num(1, 1.0),
                end: num(2, 5.0),
                step: None,
                body: vec![],
            },
        }];
        let err = lower_body(&body, &mut w, &mut map, &mut scope).unwrap_err();
        assert_eq!(
            err,
            TranspileError::InvalidIdentifier {
                block: id(0),
                name: "end".to_string(),
            }
        );
    }
}
