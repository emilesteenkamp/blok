//! Integration tests that don't just check the generated Lua *text* — they
//! actually run it through piccolo (the VM `runtime` will embed) and check
//! what it does, the same way the block-ir integration tests caught a real
//! bug by exercising the whole thing rather than individual pieces.

use std::cell::RefCell;
use std::rc::Rc;

use block_ir::{
    BinaryOp, BlockIdGenerator, Expression, ExpressionKind, Program, Statement, StatementKind,
    SwitchCase,
};
use piccolo::{Callback, CallbackReturn, Closure, Executor, Lua, Value};

/// Transpiles `program`, runs the result through piccolo with `print`
/// captured and `read_input` stubbed to always return `input`, and returns
/// each `print` call as one joined (tab-separated, matching Lua's own
/// `print`) line of output.
fn run(program: &Program, input: &str) -> Vec<String> {
    let output = lua_transpiler::transpile(program).expect("transpile should succeed");

    let captured = Rc::new(RefCell::new(Vec::<String>::new()));
    let captured_for_print = captured.clone();
    let input = input.to_string();

    let mut lua = Lua::core();
    let executor = lua
        .try_enter(|ctx| {
            let print_fn = Callback::from_fn(&ctx, move |_ctx, _, mut stack| {
                let line = (0..stack.len())
                    .map(|i| stack.get(i).to_string())
                    .collect::<Vec<_>>()
                    .join("\t");
                captured_for_print.borrow_mut().push(line);
                stack.clear();
                Ok(CallbackReturn::Return)
            });
            ctx.set_global("print", print_fn)?;

            let input = input.clone();
            let read_input_fn = Callback::from_fn(&ctx, move |ctx, _, mut stack| {
                stack.clear();
                stack.push_back(Value::String(piccolo::String::from_slice(&ctx, &input)));
                Ok(CallbackReturn::Return)
            });
            ctx.set_global("read_input", read_input_fn)?;

            let closure = Closure::load(ctx, None, output.source.as_bytes())
                .unwrap_or_else(|e| panic!("generated Lua failed to parse:\n{}\n\n{e}", output.source));
            Ok(ctx.stash(Executor::start(ctx, closure.into(), ())))
        })
        .unwrap();

    lua.execute::<()>(&executor)
        .unwrap_or_else(|e| panic!("generated Lua failed to run:\n{}\n\n{e}", output.source));

    let result = captured.borrow().clone();
    result
}

fn id_gen() -> BlockIdGenerator {
    BlockIdGenerator::new()
}

fn num(ids: &mut BlockIdGenerator, n: f64) -> Expression {
    Expression {
        id: ids.next_id(),
        kind: ExpressionKind::NumberLiteral(n),
    }
}

fn string(ids: &mut BlockIdGenerator, s: &str) -> Expression {
    Expression {
        id: ids.next_id(),
        kind: ExpressionKind::StringLiteral(s.to_string()),
    }
}

fn var(ids: &mut BlockIdGenerator, name: &str) -> Expression {
    Expression {
        id: ids.next_id(),
        kind: ExpressionKind::Variable(name.to_string()),
    }
}

#[test]
fn for_loop_running_total_matches_hand_computed_sum() {
    let mut ids = id_gen();
    let program = Program {
        body: vec![
            Statement {
                id: ids.next_id(),
                kind: StatementKind::VariableDecl {
                    name: "total".to_string(),
                    value: num(&mut ids, 0.0),
                },
            },
            Statement {
                id: ids.next_id(),
                kind: StatementKind::ForLoop {
                    variable: "i".to_string(),
                    start: num(&mut ids, 1.0),
                    end: num(&mut ids, 5.0),
                    step: None,
                    body: vec![Statement {
                        id: ids.next_id(),
                        kind: StatementKind::Assign {
                            name: "total".to_string(),
                            value: Expression {
                                id: ids.next_id(),
                                kind: ExpressionKind::BinaryOp {
                                    op: BinaryOp::Add,
                                    left: Box::new(var(&mut ids, "total")),
                                    right: Box::new(var(&mut ids, "i")),
                                },
                            },
                        },
                    }],
                },
            },
            Statement {
                id: ids.next_id(),
                kind: StatementKind::Print {
                    value: var(&mut ids, "total"),
                },
            },
        ],
    };

    assert_eq!(run(&program, ""), vec!["15"]);
}

#[test]
fn if_else_picks_the_right_branch() {
    let mut ids = id_gen();
    let program = Program {
        body: vec![Statement {
            id: ids.next_id(),
            kind: StatementKind::If {
                condition: Expression {
                    id: ids.next_id(),
                    kind: ExpressionKind::BinaryOp {
                        op: BinaryOp::Gt,
                        left: Box::new(num(&mut ids, 2.0)),
                        right: Box::new(num(&mut ids, 1.0)),
                    },
                },
                then_body: vec![Statement {
                    id: ids.next_id(),
                    kind: StatementKind::Print {
                        value: string(&mut ids, "yes"),
                    },
                }],
                else_body: vec![Statement {
                    id: ids.next_id(),
                    kind: StatementKind::Print {
                        value: string(&mut ids, "no"),
                    },
                }],
            },
        }],
    };

    assert_eq!(run(&program, ""), vec!["yes"]);
}

#[test]
fn switch_falls_through_to_default() {
    let mut ids = id_gen();
    let program = Program {
        body: vec![Statement {
            id: ids.next_id(),
            kind: StatementKind::Switch {
                subject: num(&mut ids, 3.0),
                cases: vec![
                    SwitchCase {
                        value: num(&mut ids, 1.0),
                        body: vec![Statement {
                            id: ids.next_id(),
                            kind: StatementKind::Print {
                                value: string(&mut ids, "one"),
                            },
                        }],
                    },
                    SwitchCase {
                        value: num(&mut ids, 2.0),
                        body: vec![Statement {
                            id: ids.next_id(),
                            kind: StatementKind::Print {
                                value: string(&mut ids, "two"),
                            },
                        }],
                    },
                ],
                default: vec![Statement {
                    id: ids.next_id(),
                    kind: StatementKind::Print {
                        value: string(&mut ids, "other"),
                    },
                }],
            },
        }],
    };

    assert_eq!(run(&program, ""), vec!["other"]);
}

#[test]
fn list_append_get_and_length_round_trip_with_zero_based_indexing() {
    let mut ids = id_gen();
    let program = Program {
        body: vec![
            Statement {
                id: ids.next_id(),
                kind: StatementKind::VariableDecl {
                    name: "xs".to_string(),
                    value: Expression {
                        id: ids.next_id(),
                        kind: ExpressionKind::ListLiteral(vec![
                            num(&mut ids, 10.0),
                            num(&mut ids, 20.0),
                        ]),
                    },
                },
            },
            Statement {
                id: ids.next_id(),
                kind: StatementKind::ListAppend {
                    list: var(&mut ids, "xs"),
                    value: num(&mut ids, 30.0),
                },
            },
            Statement {
                id: ids.next_id(),
                kind: StatementKind::Print {
                    // index 0 -> first element, per blok's 0-based convention
                    value: Expression {
                        id: ids.next_id(),
                        kind: ExpressionKind::ListGet {
                            list: Box::new(var(&mut ids, "xs")),
                            index: Box::new(num(&mut ids, 0.0)),
                        },
                    },
                },
            },
            Statement {
                id: ids.next_id(),
                kind: StatementKind::Print {
                    value: Expression {
                        id: ids.next_id(),
                        kind: ExpressionKind::ListLength {
                            list: Box::new(var(&mut ids, "xs")),
                        },
                    },
                },
            },
        ],
    };

    assert_eq!(run(&program, ""), vec!["10", "3"]);
}

#[test]
fn map_set_get_and_length_via_the_polyfill() {
    let mut ids = id_gen();
    let program = Program {
        body: vec![
            Statement {
                id: ids.next_id(),
                kind: StatementKind::VariableDecl {
                    name: "ages".to_string(),
                    value: Expression {
                        id: ids.next_id(),
                        kind: ExpressionKind::MapLiteral(vec![(
                            string(&mut ids, "emile"),
                            num(&mut ids, 30.0),
                        )]),
                    },
                },
            },
            Statement {
                id: ids.next_id(),
                kind: StatementKind::MapSet {
                    map: var(&mut ids, "ages"),
                    key: string(&mut ids, "ana"),
                    value: num(&mut ids, 25.0),
                },
            },
            Statement {
                id: ids.next_id(),
                kind: StatementKind::Print {
                    value: Expression {
                        id: ids.next_id(),
                        kind: ExpressionKind::MapGet {
                            map: Box::new(var(&mut ids, "ages")),
                            key: Box::new(string(&mut ids, "ana")),
                        },
                    },
                },
            },
            Statement {
                id: ids.next_id(),
                kind: StatementKind::Print {
                    value: Expression {
                        id: ids.next_id(),
                        kind: ExpressionKind::MapLength {
                            map: Box::new(var(&mut ids, "ages")),
                        },
                    },
                },
            },
        ],
    };

    assert_eq!(run(&program, ""), vec!["25", "2"]);
}

#[test]
fn math_and_strings_packages_call_the_real_piccolo_stdlib() {
    let mut ids = id_gen();
    let program = Program {
        body: vec![
            Statement {
                id: ids.next_id(),
                kind: StatementKind::Print {
                    value: Expression {
                        id: ids.next_id(),
                        kind: ExpressionKind::PackageCall {
                            package: "math".to_string(),
                            function: "sqrt".to_string(),
                            args: vec![num(&mut ids, 16.0)],
                        },
                    },
                },
            },
            Statement {
                id: ids.next_id(),
                kind: StatementKind::Print {
                    value: Expression {
                        id: ids.next_id(),
                        kind: ExpressionKind::PackageCall {
                            package: "strings".to_string(),
                            function: "concat".to_string(),
                            args: vec![string(&mut ids, "blo"), string(&mut ids, "k")],
                        },
                    },
                },
            },
            Statement {
                id: ids.next_id(),
                kind: StatementKind::Print {
                    value: Expression {
                        id: ids.next_id(),
                        kind: ExpressionKind::PackageCall {
                            package: "strings".to_string(),
                            function: "upper".to_string(),
                            args: vec![string(&mut ids, "blok")],
                        },
                    },
                },
            },
        ],
    };

    assert_eq!(run(&program, ""), vec!["4", "blok", "BLOK"]);
}

#[test]
fn read_input_returns_the_host_provided_string() {
    let mut ids = id_gen();
    let program = Program {
        body: vec![Statement {
            id: ids.next_id(),
            kind: StatementKind::Print {
                value: Expression {
                    id: ids.next_id(),
                    kind: ExpressionKind::ReadInput,
                },
            },
        }],
    };

    assert_eq!(run(&program, "hello"), vec!["hello"]);
}

#[test]
fn source_map_points_at_the_line_each_statement_was_written_to() {
    let mut ids = id_gen();
    let decl_id = ids.next_id();
    let print_id = ids.next_id();
    let program = Program {
        body: vec![
            Statement {
                id: decl_id,
                kind: StatementKind::VariableDecl {
                    name: "x".to_string(),
                    value: num(&mut ids, 1.0),
                },
            },
            Statement {
                id: print_id,
                kind: StatementKind::Print {
                    value: var(&mut ids, "x"),
                },
            },
        ],
    };

    let output = lua_transpiler::transpile(&program).unwrap();
    assert_eq!(output.source, "local x = 1\nprint(x)\n");
    assert_eq!(output.source_map.line_for(decl_id), Some(1));
    assert_eq!(output.source_map.line_for(print_id), Some(2));
    assert_eq!(output.source_map.block_at(2), Some(print_id));
}
