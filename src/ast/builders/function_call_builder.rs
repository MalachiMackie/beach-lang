use crate::{
    ast::node::{Expression, FunctionCall, FunctionId},
    token_stream::token::{Token, TokenSource},
};

use super::expression_builder::ExpressionBuilder;

#[derive(Default)]
pub struct FunctionCallBuilder {
    pub function_id: Option<FunctionId>,
    pub parameters: Option<Vec<Expression>>,
    pub comma_tokens: Vec<TokenSource>,
    pub function_identifier_token: Option<TokenSource>,
    pub left_parenthesis_token: Option<TokenSource>,
    pub right_parenthesis_token: Option<TokenSource>,
}

impl FunctionCallBuilder {
    pub fn function_id(
        mut self,
        function_id: &str,
        function_identifier_token: TokenSource,
        left_parenthesis_token: TokenSource,
        right_parenthesis_token: TokenSource,
    ) -> Self {
        self.function_id = Some(FunctionId(function_id.to_owned()));
        self.function_identifier_token = Some(function_identifier_token);
        self.left_parenthesis_token = Some(left_parenthesis_token);
        self.right_parenthesis_token = Some(right_parenthesis_token);
        self
    }

    pub fn first_parameter(
        mut self,
        expression_fn: impl FnOnce(ExpressionBuilder) -> Expression,
    ) -> Self {
        if self.parameters.is_some() {
            panic!("Cannot set first parameters when parameters have already been set");
        }

        let expression = expression_fn(ExpressionBuilder {});

        self.parameters = Some(vec![expression]);
        self
    }

    pub fn parameter(
        mut self,
        expression_fn: impl FnOnce(ExpressionBuilder) -> Expression,
        comma_token: TokenSource,
    ) -> Self {
        let expression = expression_fn(ExpressionBuilder {});

        let Some(parameters) = &mut self.parameters else {
            panic!("Must set first parameter");
        };

        parameters.push(expression);
        self.comma_tokens.push(comma_token);
        self
    }

    pub fn no_parameters(mut self) -> Self {
        self.parameters = Some(Vec::new());
        self
    }

    pub fn build(self) -> FunctionCall {
        FunctionCall {
            function_id: self.function_id.expect("function id to be set"),
            parameters: self.parameters.expect("parameters to be set"),
            comma_tokens: self.comma_tokens,
            function_id_token: self
                .function_identifier_token
                .expect("function_id_token to be set"),
            left_parenthesis_token: self
                .left_parenthesis_token
                .expect("left_parenthesis_token to be set"),
            right_parenthesis_token: self
                .right_parenthesis_token
                .expect("right_parenthesis_token to be set"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::node::FunctionId,
        token_stream::token::{Token, TokenSource},
    };

    use super::*;

    #[test]
    fn function_call_no_parameters() {
        let result = FunctionCallBuilder::default()
            .function_id(
                "my_function",
                TokenSource::dummy(Token::Identifier("my_function".to_owned())),
                TokenSource::dummy_left_parenthesis(),
                TokenSource::dummy_right_parenthesis(),
            )
            .no_parameters()
            .build();

        let expected = FunctionCall {
            function_id: FunctionId("my_function".to_owned()),
            parameters: Vec::new(),
            comma_tokens: Vec::new(),
            function_id_token: TokenSource::dummy(Token::Identifier("my_identifier".to_owned())),
            left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

            right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
        };

        assert_eq!(result, expected)
    }

    #[test]
    fn function_call_parameter() {
        let actual = FunctionCallBuilder::default()
            .function_id(
                "my_function",
                TokenSource::dummy(Token::Identifier("my_function".to_owned())),
                TokenSource::dummy_left_parenthesis(),
                TokenSource::dummy_right_parenthesis(),
            )
            .first_parameter(|param| param.value_literal(true.into(), TokenSource::dummy_true()))
            .parameter(
                |param| param.value_literal(10.into(), TokenSource::dummy_uint(10)),
                TokenSource::dummy_comma(),
            )
            .build();

        let expected = FunctionCall {
            function_id: FunctionId("my_function".to_owned()),

            parameters: vec![
                (true, TokenSource::dummy_true()).into(),
                (10, TokenSource::dummy_uint(10)).into(),
            ],
            comma_tokens: Vec::new(),
            function_id_token: TokenSource::dummy(Token::Identifier("my_function".to_owned())),
            left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

            right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
        };

        assert_eq!(actual, expected);
    }
}
