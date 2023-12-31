use crate::ast::node::{BinaryOperation, Expression, Operation, UnaryOperation};

use super::expression_builder::ExpressionBuilder;

#[derive(Default)]
pub struct OperationBuilder {}

impl OperationBuilder {
    pub fn not<TExpressionFn: Fn(ExpressionBuilder) -> Expression>(
        self,
        expression_fn: TExpressionFn,
    ) -> Operation {
        Operation::Unary {
            operation: UnaryOperation::Not,
            value: Box::new(expression_fn(ExpressionBuilder {})),
        }
    }

    pub fn greater_than(
        self,
        left_fn: impl Fn(ExpressionBuilder) -> Expression,
        right_fn: impl Fn(ExpressionBuilder) -> Expression,
    ) -> Operation {
        Operation::Binary {
            operation: BinaryOperation::GreaterThan,
            left: Box::new(left_fn(ExpressionBuilder {})),
            right: Box::new(right_fn(ExpressionBuilder {})),
        }
    }

    pub fn plus(
        self,
        left_fn: impl Fn(ExpressionBuilder) -> Expression,
        right_fn: impl Fn(ExpressionBuilder) -> Expression,
    ) -> Operation {
        Operation::Binary {
            operation: BinaryOperation::Plus,
            left: Box::new(left_fn(ExpressionBuilder {})),
            right: Box::new(right_fn(ExpressionBuilder {})),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::node::{BoolValue, UIntValue, Value};

    use super::*;

    #[test]

    fn not_operation() {
        let result = OperationBuilder::default().not(|not_expression_builder| {
            not_expression_builder.value_literal(Value::Boolean(BoolValue(true)))
        });

        let expected = Operation::Unary {
            operation: UnaryOperation::Not,
            value: Box::new(Expression::ValueLiteral(Value::Boolean(BoolValue(true)))),
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn greater_than_operation() {
        let result = OperationBuilder::default().greater_than(
            |left| left.value_literal(Value::UInt(UIntValue(10))),
            |right| right.value_literal(Value::UInt(UIntValue(12))),
        );

        let expected = Operation::Binary {
            operation: BinaryOperation::GreaterThan,
            left: Box::new(Expression::ValueLiteral(Value::UInt(UIntValue(10)))),
            right: Box::new(Expression::ValueLiteral(Value::UInt(UIntValue(12)))),
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn plus_than_operation() {
        let result = OperationBuilder::default().plus(
            |left| left.value_literal(Value::UInt(UIntValue(10))),
            |right| right.value_literal(Value::UInt(UIntValue(12))),
        );

        let expected = Operation::Binary {
            operation: BinaryOperation::Plus,
            left: Box::new(Expression::ValueLiteral(Value::UInt(UIntValue(10)))),
            right: Box::new(Expression::ValueLiteral(Value::UInt(UIntValue(12)))),
        };

        assert_eq!(result, expected);
    }
}
