//! End-to-end tests: real `block-ir` programs, transpiled by the real
//! `lua-transpiler`, driven through the real `runtime::Runtime` — the
//! same pipeline the app shell will use, minus the UI.

use block_ir::{
    BinaryOp, BlockIdGenerator, Expression, ExpressionKind, Program, Statement, StatementKind,
};
use runtime::Runtime;
use runtime_protocol::{DebugCommand, RunEvent, Value};

fn start(program: &Program) -> Runtime {
    let output = lua_transpiler::transpile(program).expect("transpile should succeed");
    Runtime::start(&output.source).expect("generated Lua should load")
}

#[test]
fn continue_runs_to_completion_and_reports_output() {
    let mut ids = BlockIdGenerator::new();
    let program = Program {
        body: vec![Statement {
            id: ids.next_id(),
            kind: StatementKind::Print {
                value: Expression {
                    id: ids.next_id(),
                    kind: ExpressionKind::StringLiteral("hello".to_string()),
                },
            },
        }],
    };

    let mut runtime = start(&program);
    assert_eq!(
        runtime.send(DebugCommand::Continue),
        vec![RunEvent::Output("hello".to_string()), RunEvent::Finished]
    );
}

#[test]
fn step_pauses_at_each_statement_reporting_locals_in_scope_so_far() {
    let mut ids = BlockIdGenerator::new();
    let decl_total = ids.next_id();
    let decl_count = ids.next_id();
    let program = Program {
        body: vec![
            Statement {
                id: decl_total,
                kind: StatementKind::VariableDecl {
                    name: "total".to_string(),
                    value: Expression {
                        id: ids.next_id(),
                        kind: ExpressionKind::NumberLiteral(0.0),
                    },
                },
            },
            Statement {
                id: decl_count,
                kind: StatementKind::VariableDecl {
                    name: "count".to_string(),
                    value: Expression {
                        id: ids.next_id(),
                        kind: ExpressionKind::NumberLiteral(1.0),
                    },
                },
            },
        ],
    };

    let mut runtime = start(&program);

    assert_eq!(
        runtime.send(DebugCommand::Step),
        vec![RunEvent::Paused {
            block: decl_total,
            locals: vec![],
        }]
    );
    assert_eq!(
        runtime.send(DebugCommand::Step),
        vec![RunEvent::Paused {
            block: decl_count,
            locals: vec![("total".to_string(), Value::Number(0.0))],
        }]
    );
    assert_eq!(runtime.send(DebugCommand::Step), vec![RunEvent::Finished]);
}

#[test]
fn breakpoint_pauses_execution_and_continue_resumes_past_it() {
    let mut ids = BlockIdGenerator::new();
    let decl_total = ids.next_id();
    let decl_flag = ids.next_id();
    let print_total = ids.next_id();
    let program = Program {
        body: vec![
            Statement {
                id: decl_total,
                kind: StatementKind::VariableDecl {
                    name: "total".to_string(),
                    value: Expression {
                        id: ids.next_id(),
                        kind: ExpressionKind::NumberLiteral(0.0),
                    },
                },
            },
            Statement {
                id: decl_flag,
                kind: StatementKind::VariableDecl {
                    name: "flag".to_string(),
                    value: Expression {
                        id: ids.next_id(),
                        kind: ExpressionKind::NumberLiteral(1.0),
                    },
                },
            },
            Statement {
                id: print_total,
                kind: StatementKind::Print {
                    value: Expression {
                        id: ids.next_id(),
                        kind: ExpressionKind::Variable("total".to_string()),
                    },
                },
            },
        ],
    };

    let mut runtime = start(&program);
    runtime.send(DebugCommand::SetBreakpoints(vec![decl_flag]));

    assert_eq!(
        runtime.send(DebugCommand::Continue),
        vec![RunEvent::Paused {
            block: decl_flag,
            locals: vec![("total".to_string(), Value::Number(0.0))],
        }]
    );
    assert_eq!(
        runtime.send(DebugCommand::Continue),
        vec![RunEvent::Output("0".to_string()), RunEvent::Finished]
    );
}

#[test]
fn read_input_suspends_until_input_is_provided() {
    let mut ids = BlockIdGenerator::new();
    let program = Program {
        body: vec![
            Statement {
                id: ids.next_id(),
                kind: StatementKind::VariableDecl {
                    name: "name".to_string(),
                    value: Expression {
                        id: ids.next_id(),
                        kind: ExpressionKind::ReadInput,
                    },
                },
            },
            Statement {
                id: ids.next_id(),
                kind: StatementKind::Print {
                    value: Expression {
                        id: ids.next_id(),
                        kind: ExpressionKind::Variable("name".to_string()),
                    },
                },
            },
        ],
    };

    let mut runtime = start(&program);

    assert_eq!(
        runtime.send(DebugCommand::Continue),
        vec![RunEvent::InputRequested]
    );
    assert_eq!(
        runtime.send(DebugCommand::ProvideInput("Ana".to_string())),
        vec![RunEvent::Output("Ana".to_string()), RunEvent::Finished]
    );
}

#[test]
fn loop_with_no_statements_in_its_body_hits_the_instruction_limit() {
    let mut ids = BlockIdGenerator::new();
    let program = Program {
        body: vec![Statement {
            id: ids.next_id(),
            kind: StatementKind::WhileLoop {
                condition: Expression {
                    id: ids.next_id(),
                    kind: ExpressionKind::BoolLiteral(true),
                },
                body: vec![],
            },
        }],
    };

    let mut runtime = start(&program);
    let events = runtime.send(DebugCommand::Continue);
    assert_eq!(events.len(), 1);
    assert!(matches!(&events[0], RunEvent::Error(msg) if msg.contains("instruction limit")));
}

#[test]
fn binary_op_result_shows_up_in_a_later_pause() {
    let mut ids = BlockIdGenerator::new();
    let decl = ids.next_id();
    let after = ids.next_id();
    let program = Program {
        body: vec![
            Statement {
                id: decl,
                kind: StatementKind::VariableDecl {
                    name: "total".to_string(),
                    value: Expression {
                        id: ids.next_id(),
                        kind: ExpressionKind::BinaryOp {
                            op: BinaryOp::Add,
                            left: Box::new(Expression {
                                id: ids.next_id(),
                                kind: ExpressionKind::NumberLiteral(2.0),
                            }),
                            right: Box::new(Expression {
                                id: ids.next_id(),
                                kind: ExpressionKind::NumberLiteral(3.0),
                            }),
                        },
                    },
                },
            },
            Statement {
                id: after,
                kind: StatementKind::Print {
                    value: Expression {
                        id: ids.next_id(),
                        kind: ExpressionKind::Variable("total".to_string()),
                    },
                },
            },
        ],
    };

    let mut runtime = start(&program);
    runtime.send(DebugCommand::SetBreakpoints(vec![after]));
    assert_eq!(
        runtime.send(DebugCommand::Continue),
        vec![RunEvent::Paused {
            block: after,
            locals: vec![("total".to_string(), Value::Number(5.0))],
        }]
    );
}

