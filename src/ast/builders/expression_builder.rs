use crate::{
    ast::node::{Expression, FunctionCall, Operation, Value},
    token_stream::token::TokenSource,
};

use super::{function_call_builder::FunctionCallBuilder, operation_builder::OperationBuilder};

#[derive(Default)]
pub struct ExpressionBuilder {}

impl ExpressionBuilder {
    pub fn function_call(
        self,
        function_call_fn: impl FnOnce(FunctionCallBuilder) -> FunctionCall,
    ) -> Expression {
        let function_call = function_call_fn(FunctionCallBuilder::default());

        Expression::FunctionCall(function_call)
    }

    pub fn variable(self, variable_name: &str, source: TokenSource) -> Expression {
        Expression::VariableAccess(variable_name.to_owned(), source)
    }

    pub fn value_literal(self, value: Value, source: TokenSource) -> Expression {
        Expression::ValueLiteral(value, source)
    }

    pub fn operation(self, operation_fn: impl FnOnce(OperationBuilder) -> Operation) -> Expression {
        let operation = operation_fn(OperationBuilder {});

        Expression::Operation(operation)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::node::{Expression, FunctionCall, FunctionId, Operation, UnaryOperation},
        token_stream::token::{Token, TokenSource},
    };

    use super::ExpressionBuilder;

    #[test]
    fn function_call() {
        let actual = ExpressionBuilder::default().function_call(|function_call| {
            function_call
                .function_id(
                    "my_function",
                    TokenSource::dummy(Token::Identifier("my_function".to_owned())),
                    TokenSource::dummy_left_parenthesis(),
                    TokenSource::dummy_right_parenthesis(),
                )
                .no_parameters()
                .build()
        });

        let expected = Expression::FunctionCall(FunctionCall {
            function_id: FunctionId("my_function".to_owned()),
            parameters: Vec::new(),
            left_parenthesis_token: TokenSource::dummy_left_parenthesis(),
            right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
            comma_tokens: Vec::new(),
            function_id_token: TokenSource::dummy(Token::Identifier("my_function".to_owned())),
        });

        assert_eq!(actual, expected);
    }

    #[test]
    fn variable() {
        let actual = ExpressionBuilder::default().variable(
            "var_name",
            TokenSource::dummy(Token::Identifier("var_name".to_owned())),
        );

        let expected = Expression::VariableAccess(
            "var_name".to_owned(),
            TokenSource::dummy(Token::Identifier("var_name".to_owned())),
        );

        assert_eq!(actual, expected);
    }

    #[test]
    fn value_literal() {
        let actual =
            ExpressionBuilder::default().value_literal(true.into(), TokenSource::dummy_true());

        let expected = (true, TokenSource::dummy_true()).into();

        assert_eq!(actual, expected);
    }

    #[test]
    fn operation() {
        let actual = ExpressionBuilder::default().operation(|operation| {
            operation.not(
                |not| not.value_literal(true.into(), TokenSource::dummy_true()),
                TokenSource::dummy(Token::NotOperator),
            )
        });

        let expected = Expression::Operation(Operation::Unary {
            operation: UnaryOperation::Not,
            value: Box::new((true, TokenSource::dummy_true()).into()),
            operator_token: TokenSource::dummy(Token::NotOperator),
        });

        assert_eq!(actual, expected);
    }
}
