use crate::{
    ast::node::{BinaryOperation, Expression, Operation, UnaryOperation},
    token_stream::token::TokenSource,
};

use super::expression_builder::ExpressionBuilder;

#[derive(Default)]
pub struct OperationBuilder {}

impl OperationBuilder {
    pub fn not<TExpressionFn: FnOnce(ExpressionBuilder) -> Expression>(
        self,
        expression_fn: TExpressionFn,
        operator_token: TokenSource,
    ) -> Operation {
        Operation::Unary {
            operation: UnaryOperation::Not,
            value: Box::new(expression_fn(ExpressionBuilder {})),
            operator_token,
        }
    }

    pub fn greater_than(
        self,
        left_fn: impl FnOnce(ExpressionBuilder) -> Expression,
        right_fn: impl FnOnce(ExpressionBuilder) -> Expression,
        operator_token: TokenSource,
    ) -> Operation {
        Operation::Binary {
            operation: BinaryOperation::GreaterThan,
            left: Box::new(left_fn(ExpressionBuilder {})),
            right: Box::new(right_fn(ExpressionBuilder {})),
            operator_token,
        }
    }

    pub fn plus(
        self,
        left_fn: impl FnOnce(ExpressionBuilder) -> Expression,
        right_fn: impl FnOnce(ExpressionBuilder) -> Expression,
        operator_token: TokenSource,
    ) -> Operation {
        Operation::Binary {
            operation: BinaryOperation::Plus,
            left: Box::new(left_fn(ExpressionBuilder {})),
            right: Box::new(right_fn(ExpressionBuilder {})),
            operator_token,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::token_stream::token::{Token, TokenSource};

    use super::*;

    #[test]
    fn not_operation() {
        let result = OperationBuilder::default().not(
            |not_expression_builder| {
                not_expression_builder.value_literal(true.into(), TokenSource::dummy_true())
            },
            TokenSource::dummy(Token::NotOperator),
        );

        let expected = Operation::Unary {
            operation: UnaryOperation::Not,
            value: Box::new((true, TokenSource::dummy_true()).into()),
            operator_token: TokenSource::dummy(Token::NotOperator),
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn greater_than_operation() {
        let result = OperationBuilder::default().greater_than(
            |left| left.value_literal(10.into(), TokenSource::dummy_uint(10)),
            |right| right.value_literal(12.into(), TokenSource::dummy_uint(12)),
            TokenSource::dummy(Token::RightAngle),
        );

        let expected = Operation::Binary {
            operation: BinaryOperation::GreaterThan,
            left: Box::new((10, TokenSource::dummy_uint(10)).into()),
            right: Box::new((12, TokenSource::dummy_uint(12)).into()),
            operator_token: TokenSource::dummy(Token::RightAngle),
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn plus_than_operation() {
        let result = OperationBuilder::default().plus(
            |left| left.value_literal(10.into(), TokenSource::dummy_uint(10)),
            |right| right.value_literal(12.into(), TokenSource::dummy_uint(12)),
            TokenSource::dummy(Token::PlusOperator)
        );

        let expected = Operation::Binary {
            operation: BinaryOperation::Plus,
            left: Box::new((10, TokenSource::dummy_uint(10)).into()),
            right: Box::new((12, TokenSource::dummy_uint(12)).into()),
            operator_token: TokenSource::dummy(Token::PlusOperator),
        };

        assert_eq!(result, expected);
    }
}
