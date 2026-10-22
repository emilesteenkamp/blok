//! End-to-end trees built the way `block-editor` will build them: via
//! `BlockIdGenerator` rather than hand-picked ids.

use block_ir::{
    validate, BinaryOp, BlockIdGenerator, Expression, ExpressionKind, Program, Statement,
    StatementKind, SwitchCase,
};

#[test]
fn for_loop_printing_running_total_is_valid() {
    let mut ids = BlockIdGenerator::new();

    let program = Program {
        body: vec![
            Statement {
                id: ids.next_id(),
                kind: StatementKind::VariableDecl {
                    name: "total".to_string(),
                    value: Expression {
                        id: ids.next_id(),
                        kind: ExpressionKind::NumberLiteral(0.0),
                    },
                },
            },
            Statement {
                id: ids.next_id(),
                kind: StatementKind::ForLoop {
                    variable: "i".to_string(),
                    start: Expression {
                        id: ids.next_id(),
                        kind: ExpressionKind::NumberLiteral(1.0),
                    },
                    end: Expression {
                        id: ids.next_id(),
                        kind: ExpressionKind::NumberLiteral(5.0),
                    },
                    step: None,
                    body: vec![
                        Statement {
                            id: ids.next_id(),
                            kind: StatementKind::Assign {
                                name: "total".to_string(),
                                value: Expression {
                                    id: ids.next_id(),
                                    kind: ExpressionKind::BinaryOp {
                                        op: BinaryOp::Add,
                                        left: Box::new(Expression {
                                            id: ids.next_id(),
                                            kind: ExpressionKind::Variable("total".to_string()),
                                        }),
                                        right: Box::new(Expression {
                                            id: ids.next_id(),
                                            kind: ExpressionKind::Variable("i".to_string()),
                                        }),
                                    },
                                },
                            },
                        },
                        Statement {
                            id: ids.next_id(),
                            kind: StatementKind::Print {
                                value: Expression {
                                    id: ids.next_id(),
                                    kind: ExpressionKind::Variable("total".to_string()),
                                },
                            },
                        },
                    ],
                },
            },
        ],
    };

    assert_eq!(validate(&program), Ok(()));
}

#[test]
fn list_build_and_append_is_valid() {
    let mut ids = BlockIdGenerator::new();

    let program = Program {
        body: vec![
            Statement {
                id: ids.next_id(),
                kind: StatementKind::VariableDecl {
                    name: "scores".to_string(),
                    value: Expression {
                        id: ids.next_id(),
                        kind: ExpressionKind::ListLiteral(vec![
                            Expression {
                                id: ids.next_id(),
                                kind: ExpressionKind::NumberLiteral(1.0),
                            },
                            Expression {
                                id: ids.next_id(),
                                kind: ExpressionKind::NumberLiteral(2.0),
                            },
                        ]),
                    },
                },
            },
            Statement {
                id: ids.next_id(),
                kind: StatementKind::ListAppend {
                    list: Expression {
                        id: ids.next_id(),
                        kind: ExpressionKind::Variable("scores".to_string()),
                    },
                    value: Expression {
                        id: ids.next_id(),
                        kind: ExpressionKind::NumberLiteral(3.0),
                    },
                },
            },
            Statement {
                id: ids.next_id(),
                kind: StatementKind::Print {
                    value: Expression {
                        id: ids.next_id(),
                        kind: ExpressionKind::ListLength {
                            list: Box::new(Expression {
                                id: ids.next_id(),
                                kind: ExpressionKind::Variable("scores".to_string()),
                            }),
                        },
                    },
                },
            },
        ],
    };

    assert_eq!(validate(&program), Ok(()));
}

#[test]
fn appending_to_a_non_list_is_rejected() {
    let mut ids = BlockIdGenerator::new();

    let program = Program {
        body: vec![Statement {
            id: ids.next_id(),
            kind: StatementKind::ListAppend {
                list: Expression {
                    id: ids.next_id(),
                    kind: ExpressionKind::NumberLiteral(1.0),
                },
                value: Expression {
                    id: ids.next_id(),
                    kind: ExpressionKind::NumberLiteral(2.0),
                },
            },
        }],
    };

    assert!(validate(&program).is_err());
}

#[test]
fn map_lookup_with_switch_and_math_package_is_valid() {
    let mut ids = BlockIdGenerator::new();

    let program = Program {
        body: vec![
            Statement {
                id: ids.next_id(),
                kind: StatementKind::VariableDecl {
                    name: "ages".to_string(),
                    value: Expression {
                        id: ids.next_id(),
                        kind: ExpressionKind::MapLiteral(vec![(
                            Expression {
                                id: ids.next_id(),
                                kind: ExpressionKind::StringLiteral("emile".to_string()),
                            },
                            Expression {
                                id: ids.next_id(),
                                kind: ExpressionKind::NumberLiteral(30.0),
                            },
                        )]),
                    },
                },
            },
            Statement {
                id: ids.next_id(),
                kind: StatementKind::Switch {
                    subject: Expression {
                        id: ids.next_id(),
                        kind: ExpressionKind::PackageCall {
                            package: "math".to_string(),
                            function: "floor".to_string(),
                            args: vec![Expression {
                                id: ids.next_id(),
                                kind: ExpressionKind::MapGet {
                                    map: Box::new(Expression {
                                        id: ids.next_id(),
                                        kind: ExpressionKind::Variable("ages".to_string()),
                                    }),
                                    key: Box::new(Expression {
                                        id: ids.next_id(),
                                        kind: ExpressionKind::StringLiteral("emile".to_string()),
                                    }),
                                },
                            }],
                        },
                    },
                    cases: vec![SwitchCase {
                        value: Expression {
                            id: ids.next_id(),
                            kind: ExpressionKind::NumberLiteral(30.0),
                        },
                        body: vec![Statement {
                            id: ids.next_id(),
                            kind: StatementKind::Print {
                                value: Expression {
                                    id: ids.next_id(),
                                    kind: ExpressionKind::StringLiteral("thirty".to_string()),
                                },
                            },
                        }],
                    }],
                    default: vec![],
                },
            },
        ],
    };

    assert_eq!(validate(&program), Ok(()));
}
