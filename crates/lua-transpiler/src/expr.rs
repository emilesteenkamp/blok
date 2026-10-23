//! Lowers [`block_ir::Expression`]s to Lua expression text.
//!
//! Every function here returns a self-contained, already-parenthesized-
//! where-necessary Lua expression string, so callers can embed it anywhere
//! another expression is expected without worrying about operator
//! precedence.

use block_ir::{BinaryOp, Expression, ExpressionKind, UnaryOp};

use crate::error::TranspileError;
use crate::lua::{escape_string, format_number, is_valid_identifier};
use crate::packages;

/// Lowers an expression to a Lua expression string.
pub(crate) fn lower(expr: &Expression) -> Result<String, TranspileError> {
    match &expr.kind {
        ExpressionKind::NumberLiteral(n) => Ok(format_number(*n)),
        ExpressionKind::StringLiteral(s) => Ok(escape_string(s)),
        ExpressionKind::BoolLiteral(b) => Ok(b.to_string()),
        ExpressionKind::Variable(name) => lower_identifier(expr, name),
        ExpressionKind::BinaryOp { op, left, right } => {
            let left = lower(left)?;
            let right = lower(right)?;
            Ok(format!("({left} {} {right})", binary_op_symbol(*op)))
        }
        ExpressionKind::UnaryOp { op, operand } => {
            let operand = lower(operand)?;
            Ok(match op {
                UnaryOp::Negate => format!("(-{operand})"),
                UnaryOp::Not => format!("(not {operand})"),
            })
        }
        ExpressionKind::ListLiteral(items) => {
            let items = items.iter().map(lower).collect::<Result<Vec<_>, _>>()?;
            Ok(format!("{{{}}}", items.join(", ")))
        }
        ExpressionKind::ListGet { list, index } => {
            let list = lower(list)?;
            let index = lower(index)?;
            // block-ir indexes lists from 0; Lua tables index from 1.
            Ok(format!("({list})[({index}) + 1]"))
        }
        ExpressionKind::ListLength { list } => Ok(format!("#({})", lower(list)?)),
        ExpressionKind::MapLiteral(entries) => {
            let entries = entries
                .iter()
                .map(|(k, v)| Ok(format!("[{}] = {}", lower(k)?, lower(v)?)))
                .collect::<Result<Vec<_>, TranspileError>>()?;
            Ok(format!("{{{}}}", entries.join(", ")))
        }
        ExpressionKind::MapGet { map, key } => {
            Ok(format!("({})[{}]", lower(map)?, lower(key)?))
        }
        ExpressionKind::MapLength { map } => Ok(format!("__map_length({})", lower(map)?)),
        ExpressionKind::ReadInput => Ok("read_input()".to_string()),
        ExpressionKind::PackageCall {
            package,
            function,
            args,
        } => {
            let arg_code = args.iter().map(lower).collect::<Result<Vec<_>, _>>()?;
            packages::lower(package, function, &arg_code).ok_or_else(|| {
                TranspileError::UnknownPackageFunction {
                    block: expr.id,
                    package: package.clone(),
                    function: function.clone(),
                }
            })
        }
    }
}

fn lower_identifier(expr: &Expression, name: &str) -> Result<String, TranspileError> {
    if is_valid_identifier(name) {
        Ok(name.to_string())
    } else {
        Err(TranspileError::InvalidIdentifier {
            block: expr.id,
            name: name.to_string(),
        })
    }
}

fn binary_op_symbol(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::Div => "/",
        BinaryOp::Mod => "%",
        BinaryOp::Eq => "==",
        BinaryOp::NotEq => "~=",
        BinaryOp::Lt => "<",
        BinaryOp::LtEq => "<=",
        BinaryOp::Gt => ">",
        BinaryOp::GtEq => ">=",
        BinaryOp::And => "and",
        BinaryOp::Or => "or",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use block_ir::BlockId;

    fn id(raw: u64) -> BlockId {
        BlockId::from_raw(raw)
    }

    fn expr(kind: ExpressionKind) -> Expression {
        Expression { id: id(0), kind }
    }

    #[test]
    fn literals() {
        assert_eq!(lower(&expr(ExpressionKind::NumberLiteral(1.0))).unwrap(), "1");
        assert_eq!(
            lower(&expr(ExpressionKind::StringLiteral("hi".to_string()))).unwrap(),
            "\"hi\""
        );
        assert_eq!(lower(&expr(ExpressionKind::BoolLiteral(true))).unwrap(), "true");
    }

    #[test]
    fn variable_reference() {
        assert_eq!(
            lower(&expr(ExpressionKind::Variable("total".to_string()))).unwrap(),
            "total"
        );
    }

    #[test]
    fn invalid_variable_name_is_an_error() {
        let e = expr(ExpressionKind::Variable("end".to_string()));
        assert_eq!(
            lower(&e),
            Err(TranspileError::InvalidIdentifier {
                block: id(0),
                name: "end".to_string(),
            })
        );
    }

    #[test]
    fn binary_op_is_parenthesized_infix() {
        let e = expr(ExpressionKind::BinaryOp {
            op: BinaryOp::Add,
            left: Box::new(Expression {
                id: id(1),
                kind: ExpressionKind::NumberLiteral(1.0),
            }),
            right: Box::new(Expression {
                id: id(2),
                kind: ExpressionKind::NumberLiteral(2.0),
            }),
        });
        assert_eq!(lower(&e).unwrap(), "(1 + 2)");
    }

    #[test]
    fn list_get_converts_zero_based_index_to_one_based() {
        let e = expr(ExpressionKind::ListGet {
            list: Box::new(Expression {
                id: id(1),
                kind: ExpressionKind::Variable("xs".to_string()),
            }),
            index: Box::new(Expression {
                id: id(2),
                kind: ExpressionKind::NumberLiteral(0.0),
            }),
        });
        assert_eq!(lower(&e).unwrap(), "(xs)[(0) + 1]");
    }

    #[test]
    fn map_length_calls_the_polyfill() {
        let e = expr(ExpressionKind::MapLength {
            map: Box::new(Expression {
                id: id(1),
                kind: ExpressionKind::Variable("m".to_string()),
            }),
        });
        assert_eq!(lower(&e).unwrap(), "__map_length(m)");
    }

    #[test]
    fn package_call_lowers_via_the_registry() {
        let e = expr(ExpressionKind::PackageCall {
            package: "math".to_string(),
            function: "sqrt".to_string(),
            args: vec![Expression {
                id: id(1),
                kind: ExpressionKind::NumberLiteral(4.0),
            }],
        });
        assert_eq!(lower(&e).unwrap(), "math.sqrt(4)");
    }

    #[test]
    fn unknown_package_call_is_an_error() {
        let e = expr(ExpressionKind::PackageCall {
            package: "nonexistent".to_string(),
            function: "whatever".to_string(),
            args: vec![],
        });
        assert_eq!(
            lower(&e),
            Err(TranspileError::UnknownPackageFunction {
                block: id(0),
                package: "nonexistent".to_string(),
                function: "whatever".to_string(),
            })
        );
    }
}
