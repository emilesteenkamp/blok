//! Lowers [`block_ir::Statement`]s to Lua source, recording a
//! [`crate::SourceMap`] entry for each one.

use block_ir::{Statement, StatementKind};

use crate::error::TranspileError;
use crate::expr;
use crate::lua::is_valid_identifier;
use crate::source_map::SourceMap;
use crate::writer::Writer;

/// Lowers a chain of statements (a program body or a nested block),
/// writing each at the writer's current indentation.
pub(crate) fn lower_body(
    body: &[Statement],
    w: &mut Writer,
    map: &mut SourceMap,
) -> Result<(), TranspileError> {
    for stmt in body {
        lower(stmt, w, map)?;
    }
    Ok(())
}

fn lower(stmt: &Statement, w: &mut Writer, map: &mut SourceMap) -> Result<(), TranspileError> {
    map.record(stmt.id, w.current_line());
    match &stmt.kind {
        StatementKind::VariableDecl { name, value } => {
            let name = checked_identifier(stmt, name)?;
            let value = expr::lower(value)?;
            w.writeln(&format!("local {name} = {value}"));
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
            w.indented(|w| lower_body(then_body, w, map))?;
            if !else_body.is_empty() {
                w.writeln("else");
                w.indented(|w| lower_body(else_body, w, map))?;
            }
            w.writeln("end");
        }
        StatementKind::Switch {
            subject,
            cases,
            default,
        } => lower_switch(stmt, subject, cases, default, w, map)?,
        StatementKind::WhileLoop { condition, body } => {
            let condition = expr::lower(condition)?;
            w.writeln(&format!("while {condition} do"));
            w.indented(|w| lower_body(body, w, map))?;
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
            w.indented(|w| lower_body(body, w, map))?;
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

/// Lua has no `switch`. Lowered to an if/elseif chain comparing a
/// once-evaluated temporary against each (literal) case value.
fn lower_switch(
    stmt: &Statement,
    subject: &block_ir::Expression,
    cases: &[block_ir::SwitchCase],
    default: &[Statement],
    w: &mut Writer,
    map: &mut SourceMap,
) -> Result<(), TranspileError> {
    let temp = format!("__switch_{}", stmt.id.raw());
    let subject = expr::lower(subject)?;
    w.writeln(&format!("local {temp} = {subject}"));

    for (i, case) in cases.iter().enumerate() {
        let value = expr::lower(&case.value)?;
        let keyword = if i == 0 { "if" } else { "elseif" };
        w.writeln(&format!("{keyword} {temp} == {value} then"));
        w.indented(|w| lower_body(&case.body, w, map))?;
    }

    if !default.is_empty() {
        if cases.is_empty() {
            w.writeln("if true then");
        } else {
            w.writeln("else");
        }
        w.indented(|w| lower_body(default, w, map))?;
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
        lower_body(body, &mut w, &mut map).unwrap();
        (w.finish(), map)
    }

    #[test]
    fn variable_decl_emits_local() {
        let (src, map) = run(&[Statement {
            id: id(0),
            kind: StatementKind::VariableDecl {
                name: "total".to_string(),
                value: num(1, 0.0),
            },
        }]);
        assert_eq!(src, "local total = 0\n");
        assert_eq!(map.line_for(id(0)), Some(1));
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
        assert_eq!(src, "if true then\n    print(1)\nend\n");
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
        assert_eq!(
            src,
            "local __switch_0 = day\nif __switch_0 == 1 then\n    print(1)\nelse\n    print(0)\nend\n"
        );
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
        assert_eq!(src, "for i = 1, 5 do\nend\n");
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
        assert_eq!(src, "(xs)[#(xs) + 1] = 9\n");
    }

    #[test]
    fn invalid_loop_variable_name_is_an_error() {
        let mut w = Writer::new();
        let mut map = SourceMap::new();
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
        let err = lower_body(&body, &mut w, &mut map).unwrap_err();
        assert_eq!(
            err,
            TranspileError::InvalidIdentifier {
                block: id(0),
                name: "end".to_string(),
            }
        );
    }
}
