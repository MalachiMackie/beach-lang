use crate::{
    ast::node::{Ast, ElseBlock, ElseIfBlock, Expression, IfStatement, Node},
    token_stream::token::TokenSource,
};

use super::{ast_builder::AstBuilder, expression_builder::ExpressionBuilder};

pub struct IfStatementBuilder {
    check_expression: Option<Expression>,
    body: Option<Ast>,
    else_if_blocks: Vec<ElseIfBuilder>,
    else_block: Option<ElseBuilder>,
    if_keyword_token: Option<TokenSource>,
    left_parenthesis_token: Option<TokenSource>,
    right_parenthesis_token: Option<TokenSource>,
    left_curley_brace_token: Option<TokenSource>,
    right_curley_brace_token: Option<TokenSource>,
}

struct ElseIfBuilder {
    else_keyword: TokenSource,
    if_keyword: TokenSource,
    check_expression: Expression,
    left_parenthesis_token: TokenSource,
    right_parenthesis_token: TokenSource,
    left_curley_brace_token: TokenSource,
    right_curley_brace_token: TokenSource,
    ast: Ast,
}

struct ElseBuilder {
    else_keyword: TokenSource,
    left_curley_brace_token: TokenSource,
    right_curley_brace_token: TokenSource,
    ast: Ast,
}

impl IfStatementBuilder {
    pub fn new(if_keyword_token: TokenSource) -> Self {
        Self {
            check_expression: None,
            body: None,
            else_if_blocks: Vec::new(),
            else_block: None,
            if_keyword_token: Some(if_keyword_token),
            left_parenthesis_token: None,
            right_parenthesis_token: None,
            left_curley_brace_token: None,
            right_curley_brace_token: None,
        }
    }

    pub fn check_expression(
        mut self,
        expression_fn: impl FnOnce(ExpressionBuilder) -> Expression,
        left_parenthesis_token: TokenSource,
        right_parenthesis_token: TokenSource,
    ) -> Self {
        let expression = expression_fn(ExpressionBuilder {});
        self.check_expression = Some(expression);
        self.left_parenthesis_token = Some(left_parenthesis_token);
        self.right_parenthesis_token = Some(right_parenthesis_token);

        self
    }

    pub fn body(
        mut self,
        left_curley_brace_token: TokenSource,
        right_curley_brace_token: TokenSource,
        body_fn: impl FnOnce(AstBuilder) -> Ast,
    ) -> Self {
        let body = body_fn(AstBuilder::default());
        self.body = Some(body);
        self.left_curley_brace_token = Some(left_curley_brace_token);
        self.right_curley_brace_token = Some(right_curley_brace_token);

        self
    }

    pub fn else_if(
        mut self,
        check_fn: impl FnOnce(ExpressionBuilder) -> Expression,
        body_fn: impl FnOnce(AstBuilder) -> Ast,
        else_keyword: TokenSource,
        if_keyword: TokenSource,
        left_curley_brace_token: TokenSource,
        right_curley_brace_token: TokenSource,
        left_parenthesis_token: TokenSource,
        right_parenthesis_token: TokenSource,
    ) -> Self {
        let check = check_fn(ExpressionBuilder {});
        let body = body_fn(AstBuilder::default());

        self.else_if_blocks.push(ElseIfBuilder {
            else_keyword,
            left_curley_brace_token,
            right_curley_brace_token,
            ast: body,
            if_keyword,
            check_expression: check,
            left_parenthesis_token,
            right_parenthesis_token,
        });

        self
    }

    pub fn else_block(
        mut self,
        else_keyword: TokenSource,
        left_curley_brace_token: TokenSource,
        right_curley_brace_token: TokenSource,
        body_fn: impl FnOnce(AstBuilder) -> Ast,
    ) -> Self {
        let body = body_fn(AstBuilder::default());

        self.else_block = Some(ElseBuilder {
            else_keyword,
            left_curley_brace_token,
            right_curley_brace_token,
            ast: body,
        });

        self
    }

    pub fn build(self) -> Node {
        Node::IfStatement(IfStatement {
            check_expression: self.check_expression.expect("check expression to be set"),
            if_block: self.body.expect("body to be set").nodes,
            else_if_blocks: self
                .else_if_blocks
                .into_iter()
                .map(|else_if_builder| ElseIfBlock {
                    check: else_if_builder.check_expression,
                    block: else_if_builder.ast.nodes,
                    else_token: else_if_builder.else_keyword,
                    if_token: else_if_builder.if_keyword,
                    left_curley_brace_token: else_if_builder.left_curley_brace_token,
                    left_parenthesis_token: else_if_builder.left_parenthesis_token,
                    right_curley_brace_token: else_if_builder.right_curley_brace_token,
                    right_parenthesis_token: else_if_builder.right_parenthesis_token,
                })
                .collect(),
            else_block: self.else_block.map(|else_builder| ElseBlock {
                left_curley_brace_token: else_builder.left_curley_brace_token,
                nodes: else_builder.ast.nodes,
                right_curley_brace_token: else_builder.right_curley_brace_token,
            }),
            if_token: self.if_keyword_token.expect("if_token to be set"),
            left_curley_brace_token: self
                .left_curley_brace_token
                .expect("left_curley_brace_token to be set"),
            left_parenthesis_token: self
                .left_parenthesis_token
                .expect("left_parenthesis_token to be set"),
            right_curley_brace_token: self
                .right_curley_brace_token
                .expect("right_curley_brace_token to be set"),
            right_parenthesis_token: self
                .right_parenthesis_token
                .expect("right_parenthesis_token to be set"),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{
            builders::if_statement_builder::IfStatementBuilder,
            node::{ElseBlock, ElseIfBlock, FunctionCall, FunctionId, IfStatement, Node},
        },
        token_stream::token::{Token, TokenSource},
    };

    #[test]
    fn if_statement() {
        let actual = IfStatementBuilder::new(TokenSource::dummy_if())
            .check_expression(
                |check| check.value_literal(true.into(), TokenSource::dummy_true()),
                TokenSource::dummy_left_parenthesis(),
                TokenSource::dummy_right_parenthesis(),
            )
            .body(|body| {
                body.statement(|statement| {
                    statement.function_call(|function_call| {
                        function_call
                            .function_id(
                                "my_function",
                                TokenSource::dummy_function(),
                                TokenSource::dummy_left_parenthesis(),
                                TokenSource::dummy_right_parenthesis(),
                            )
                            .no_parameters()
                            .build()
                    })
                })
                .build()
            })
            .build();

        let expected = Node::IfStatement(IfStatement {
            check_expression: (true, TokenSource::dummy_true()).into(),
            if_block: vec![Node::FunctionCall(FunctionCall {
                function_id: FunctionId("my_function".to_owned()),
                parameters: Vec::new(),
                comma_tokens: Vec::new(),
                function_id_token: TokenSource::dummy(Token::Identifier("my_function".to_owned())),
                left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

                right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
            })],
            else_block: None,
            else_if_blocks: Vec::new(),
            if_token: TokenSource::dummy_if(),
            left_curley_brace_token: TokenSource::dummy_right_curley_brace(),
            left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

            right_curley_brace_token: TokenSource::dummy_right_curley_brace(),
            right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
        });

        assert_eq!(actual, expected);
    }

    #[test]
    fn else_if_statement() {
        let actual = IfStatementBuilder::new()
            .check_expression(|check| check.value_literal(true.into(), TokenSource::dummy_true()))
            .body(|body| {
                body.statement(|statement| {
                    statement.function_call(|function_call| {
                        function_call
                            .function_id(
                                "my_function",
                                TokenSource::dummy(Token::Identifier("my_function".to_owned())),
                                TokenSource::dummy_left_parenthesis(),
                                TokenSource::dummy_right_parenthesis(),
                            )
                            .no_parameters()
                            .build()
                    })
                })
                .build()
            })
            .else_if(
                |check| check.value_literal(true.into(), TokenSource::dummy_true()),
                |body| {
                    body.statement(|statement| {
                        statement.function_call(|function_call| {
                            function_call
                                .function_id(
                                    "my_function",
                                    TokenSource::dummy(Token::Identifier("my_function".to_owned())),
                                    TokenSource::dummy_left_parenthesis(),
                                    TokenSource::dummy_right_parenthesis(),
                                )
                                .no_parameters()
                                .build()
                        })
                    })
                    .build()
                },
            )
            .build();

        let expected = Node::IfStatement(IfStatement {
            check_expression: (true, TokenSource::dummy_true()).into(),
            if_block: vec![Node::FunctionCall(FunctionCall {
                function_id: FunctionId("my_function".to_owned()),
                parameters: Vec::new(),
                comma_tokens: Vec::new(),
                function_id_token: TokenSource::dummy(Token::Identifier("my_function".to_owned())),
                left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

                right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
            })],
            else_block: None,
            else_if_blocks: vec![ElseIfBlock {
                check: (true, TokenSource::dummy_true()).into(),
                block: vec![Node::FunctionCall(FunctionCall {
                    function_id: FunctionId("my_function".to_owned()),
                    parameters: Vec::new(),
                    comma_tokens: Vec::new(),
                    function_id_token: TokenSource::dummy(Token::Identifier(
                        "my_function".to_owned(),
                    )),
                    left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

                    right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
                })],
                else_token: TokenSource::dummy_else(),
                if_token: TokenSource::dummy_if(),
                left_curley_brace_token: TokenSource::dummy_right_curley_brace(),
                left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

                right_curley_brace_token: TokenSource::dummy_right_curley_brace(),
                right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
            }],
            if_token: TokenSource::dummy_if(),
            left_curley_brace_token: TokenSource::dummy_right_curley_brace(),
            left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

            right_curley_brace_token: TokenSource::dummy_right_curley_brace(),
            right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
        });

        assert_eq!(actual, expected);
    }

    #[test]
    fn else_statement() {
        let actual = IfStatementBuilder::new()
            .check_expression(|check| check.value_literal(true.into(), TokenSource::dummy_true()))
            .body(|body| {
                body.statement(|statement| {
                    statement.function_call(|function_call| {
                        function_call
                            .function_id(
                                "my_function",
                                TokenSource::dummy_function(),
                                TokenSource::dummy_left_parenthesis(),
                                TokenSource::dummy_right_parenthesis(),
                            )
                            .no_parameters()
                            .build()
                    })
                })
                .build()
            })
            .else_block(|body| {
                body.statement(|statement| {
                    statement.function_call(|function_call| {
                        function_call
                            .function_id(
                                "my_function",
                                TokenSource::dummy(Token::Identifier("my_function".to_owned())),
                                TokenSource::dummy_left_parenthesis(),
                                TokenSource::dummy_right_parenthesis(),
                            )
                            .no_parameters()
                            .build()
                    })
                })
                .build()
            })
            .build();

        let expected = Node::IfStatement(IfStatement {
            check_expression: (true, TokenSource::dummy_true()).into(),
            if_block: vec![Node::FunctionCall(FunctionCall {
                function_id: FunctionId("my_function".to_owned()),
                parameters: Vec::new(),
                comma_tokens: Vec::new(),
                function_id_token: TokenSource::dummy(Token::Identifier("my_function".to_owned())),
                left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

                right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
            })],
            else_block: Some(ElseBlock {
                nodes: vec![Node::FunctionCall(FunctionCall {
                    function_id: FunctionId("my_function".to_owned()),
                    parameters: Vec::new(),
                    comma_tokens: Vec::new(),
                    function_id_token: TokenSource::dummy(Token::Identifier(
                        "my_function".to_owned(),
                    )),
                    left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

                    right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
                })],
                left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

                right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
            }),
            else_if_blocks: Vec::new(),
            if_token: TokenSource::dummy_if(),
            left_curley_brace_token: TokenSource::dummy_right_curley_brace(),
            left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

            right_curley_brace_token: TokenSource::dummy_right_curley_brace(),
            right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
        });

        assert_eq!(actual, expected);
    }
}
