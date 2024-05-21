use crate::{
    ast::node::{Expression, FunctionCall, Node, NodeSource},
    token_stream::token::TokenSource,
};

use super::{
    expression_builder::ExpressionBuilder, function_call_builder::FunctionCallBuilder,
    if_statement_builder::IfStatementBuilder,
    variable_declaration_builder::VariableDeclarationBuilder,
};

#[derive(Default)]
pub struct StatementBuilder {
    start_token: Option<TokenSource>,
    end_token: Option<TokenSource>,
}

impl StatementBuilder {
    pub fn return_void(self) -> Node {
        let start_token = self.start_token.expect("start_token to have been set");
        let end_token = self.end_token.expect("end_token to have been set");
        let node_source = NodeSource::from_tokens(start_token, end_token);
        Node::FunctionReturn {
            return_value: None,
            source: node_source,
        }
    }

    pub fn var_declaration(
        self,
        var_declaration_fn: impl FnOnce(VariableDeclarationBuilder) -> Node,
    ) -> Node {
        var_declaration_fn(VariableDeclarationBuilder {
            var_name: None,
            var_type: None,
        })
    }

    pub fn if_statement(self, if_statement_fn: impl FnOnce(IfStatementBuilder) -> Node) -> Node {
        if_statement_fn(IfStatementBuilder::new())
    }

    pub fn function_call(
        self,
        function_call_fn: impl FnOnce(FunctionCallBuilder) -> FunctionCall,
    ) -> Node {
        Node::FunctionCall(function_call_fn(FunctionCallBuilder::default()))
    }

    pub fn return_value(self, expression: impl FnOnce(ExpressionBuilder) -> Expression) -> Node {
        Node::FunctionReturn {
            return_value: Some(expression(ExpressionBuilder {})),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{
            builders::statement_builder::StatementBuilder,
            node::{FunctionCall, FunctionId, IfStatement, Node},
        },
        token_stream::token::{Token, TokenSource},
    };

    #[test]
    fn var_declaration() {
        let actual = StatementBuilder::default().var_declaration(|var_declaration| {
            var_declaration
                .infer_type()
                .name("my_var")
                .with_assignment(|assignment| assignment.value_literal(true.into()))
        });

        let expected = Node::VariableDeclaration {
            var_type: crate::ast::node::VariableDeclarationType::Infer,
            var_name: "my_var".to_owned(),
            value: (true, TokenSource::dummy_true()).into(),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn if_statement() {
        let actual = StatementBuilder::default().if_statement(|if_statement| {
            if_statement
                .check_expression(|check| check.value_literal(true.into()))
                .body(|body| body.build())
                .build()
        });

        let expected = Node::IfStatement(IfStatement {
            check_expression: (true, TokenSource::dummy_true()).into(),
            if_block: Vec::new(),
            else_if_blocks: Vec::new(),
            else_block: None,
            if_token: TokenSource::dummy_if(),
            left_curley_brace_token: TokenSource::dummy_right_curley_brace(),
            left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

            right_curley_brace_token: TokenSource::dummy_right_curley_brace(),
            right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
        });

        assert_eq!(actual, expected);
    }

    #[test]
    fn function_call() {
        let actual = StatementBuilder::default().function_call(|function_call| {
            function_call
                .function_id("my_function")
                .no_parameters()
                .build()
        });

        let expected = Node::FunctionCall(FunctionCall {
            function_id: FunctionId("my_function".to_owned()),
            parameters: Vec::new(),
            comma_tokens: Vec::new(),
            function_id_token: TokenSource::dummy(Token::Identifier("my_function".to_owned())),
            left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

            right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
        });

        assert_eq!(actual, expected);
    }

    #[test]
    fn return_value() {
        let actual = StatementBuilder::default()
            .return_value(|return_value| return_value.value_literal(true.into()));

        let expected = Node::FunctionReturn {
            return_value: Some((true, TokenSource::dummy_true()).into()),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn return_void() {
        let actual = StatementBuilder::default().return_void();

        let expected = Node::FunctionReturn { return_value: None };

        assert_eq!(actual, expected);
    }
}
