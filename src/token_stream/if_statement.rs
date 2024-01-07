use std::collections::VecDeque;

use crate::ast::{
    builders::{
        expression_builder::ExpressionBuilder, if_statement_builder::IfStatementBuilder,
        statement_builder::StatementBuilder,
    },
    node::{Expression, Node},
};

use super::{
    expression::create_expression,
    token::{ensure_token, get_block_statements, Token, TokenStreamError},
};

pub(super) fn try_create_if_statement(
    tokens: &mut VecDeque<Token>,
) -> Result<impl FnOnce(IfStatementBuilder) -> Node, Vec<TokenStreamError>> {
    ensure_token(tokens, Token::LeftParenthesis)?;

    let check_expression = create_expression(tokens)?;

    ensure_token(tokens, Token::RightParenthesis)?;
    ensure_token(tokens, Token::LeftCurleyBrace)?;

    let statements = get_block_statements(tokens)?;
    let mut else_statements = None;
    let mut else_if_blocks = Vec::new();

    let mut found_else = false;
    loop {
        match tokens.pop_front() {
            None if found_else => {
                return Err(vec![TokenStreamError {
                    message: format!(
                        "Expected {} or {}",
                        Token::IfKeyword,
                        Token::LeftCurleyBrace
                    ),
                }])
            }
            None => {
                return Ok(build_if_statement(
                    check_expression,
                    statements,
                    else_statements,
                    else_if_blocks,
                ));
            }
            Some(Token::ElseKeyword) if found_else => {
                return Err(vec![TokenStreamError {
                    message: format!(
                        "Expected {} or {}",
                        Token::IfKeyword,
                        Token::LeftCurleyBrace
                    ),
                }])
            }
            Some(Token::ElseKeyword) => {
                found_else = true;
            }
            Some(Token::LeftCurleyBrace) if found_else => {
                else_statements = Some(get_block_statements(tokens)?);
                return Ok(build_if_statement(
                    check_expression,
                    statements,
                    else_statements,
                    else_if_blocks,
                ));
            }
            Some(Token::IfKeyword) if found_else => {
                ensure_token(tokens, Token::LeftParenthesis)?;
                let check_expression = create_expression(tokens)?;
                ensure_token(tokens, Token::RightParenthesis)?;
                ensure_token(tokens, Token::LeftCurleyBrace)?;
                let statements = get_block_statements(tokens)?;

                else_if_blocks.push((check_expression, statements));
                found_else = false;
            }
            Some(_) if found_else => {
                return Err(vec![TokenStreamError {
                    message: format!(
                        "Expected {} or {}",
                        Token::IfKeyword,
                        Token::LeftCurleyBrace
                    ),
                }])
            }
            Some(token) => {
                tokens.push_front(token);
                return Ok(build_if_statement(
                    check_expression,
                    statements,
                    else_statements,
                    else_if_blocks,
                ));
            }
        };
    }
}

fn build_if_statement(
    check_expression: Box<dyn FnOnce(ExpressionBuilder) -> Expression>,
    statements: Vec<Box<dyn FnOnce(StatementBuilder) -> Node>>,
    else_statements: Option<Vec<Box<dyn FnOnce(StatementBuilder) -> Node>>>,
    else_if_blocks: Vec<(
        Box<dyn FnOnce(ExpressionBuilder) -> Expression>,
        Vec<Box<dyn FnOnce(StatementBuilder) -> Node>>,
    )>,
) -> Box<dyn FnOnce(IfStatementBuilder) -> Node> {
    Box::new(|mut if_statement_builder: IfStatementBuilder| {
        if_statement_builder = if_statement_builder
            .check_expression(check_expression)
            .body(|mut body| {
                for statement in statements {
                    body = body.statement(statement);
                }

                body.build()
            });

        for (else_if_check, else_if_statements) in else_if_blocks {
            if_statement_builder = if_statement_builder.else_if(else_if_check, move |mut body| {
                for statement in else_if_statements {
                    body = body.statement(statement);
                }

                body.build()
            })
        }

        if let Some(else_statements) = else_statements {
            if_statement_builder = if_statement_builder.else_block(move |mut body| {
                for statement in else_statements {
                    body = body.statement(statement);
                }

                body.build()
            })
        }

        if_statement_builder.build()
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::builders::{ast_builder::AstBuilder, function_call_builder::FunctionCallBuilder},
        token_stream::token::Token,
    };

    /// if (true) { infer a = false; }
    #[test]
    fn if_statement() {
        let tokens = vec![
            Token::IfKeyword,
            Token::LeftParenthesis,
            Token::TrueKeyword,
            Token::RightParenthesis,
            Token::LeftCurleyBrace,
            Token::InferKeyword,
            Token::Identifier("a".to_owned()),
            Token::AssignmentOperator,
            Token::FalseKeyword,
            Token::SemiColon,
            Token::RightCurleyBrace,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        let expected = AstBuilder::default().statement(|statement| {
            statement.if_statement(|if_statement| {
                if_statement
                    .check_expression(|_| true.into())
                    .body(|body| {
                        body.statement(|statement| {
                            statement.var_declaration(|var_declaration| {
                                var_declaration
                                    .infer_type()
                                    .name("a")
                                    .with_assignment(|_| false.into())
                            })
                        })
                        .build()
                    })
                    .build()
            })
        });

        assert!(matches!(result, Ok(ast_builder) if ast_builder == expected));
    }

    /// if (true) { infer a = false; } else { infer b = true; }
    #[test]
    fn if_else_statement() {
        let tokens = vec![
            Token::IfKeyword,
            Token::LeftParenthesis,
            Token::TrueKeyword,
            Token::RightParenthesis,
            Token::LeftCurleyBrace,
            Token::InferKeyword,
            Token::Identifier("a".to_owned()),
            Token::AssignmentOperator,
            Token::FalseKeyword,
            Token::SemiColon,
            Token::RightCurleyBrace,
            Token::ElseKeyword,
            Token::LeftCurleyBrace,
            Token::InferKeyword,
            Token::Identifier("b".to_owned()),
            Token::AssignmentOperator,
            Token::TrueKeyword,
            Token::SemiColon,
            Token::RightCurleyBrace,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        let expected = AstBuilder::default().statement(|statement| {
            statement.if_statement(|if_statement| {
                if_statement
                    .check_expression(|_| true.into())
                    .body(|body| {
                        body.statement(|statement| {
                            statement.var_declaration(|var_declaration| {
                                var_declaration
                                    .infer_type()
                                    .name("a")
                                    .with_assignment(|_| false.into())
                            })
                        })
                        .build()
                    })
                    .else_block(|body| {
                        body.statement(|statement| {
                            statement.var_declaration(|var_declaration| {
                                var_declaration
                                    .infer_type()
                                    .name("b")
                                    .with_assignment(|_| true.into())
                            })
                        })
                        .build()
                    })
                    .build()
            })
        });

        assert!(matches!(result, Ok(ast_builder) if ast_builder == expected));
    }

    /// if (true)
    /// {
    ///     infer a = false;
    /// }
    /// else if (true)
    /// {
    ///     infer b = true;
    /// }
    /// else if (true)
    /// {
    ///     infer c = true;
    /// }
    /// else
    /// {
    ///     infer d = true;
    /// }
    #[test]
    fn if_else_if_statement() {
        let tokens = vec![
            Token::IfKeyword,
            Token::LeftParenthesis,
            Token::TrueKeyword,
            Token::RightParenthesis,
            Token::LeftCurleyBrace,
            Token::InferKeyword,
            Token::Identifier("a".to_owned()),
            Token::AssignmentOperator,
            Token::FalseKeyword,
            Token::SemiColon,
            Token::RightCurleyBrace,
            Token::ElseKeyword,
            Token::IfKeyword,
            Token::LeftParenthesis,
            Token::TrueKeyword,
            Token::RightParenthesis,
            Token::LeftCurleyBrace,
            Token::InferKeyword,
            Token::Identifier("b".to_owned()),
            Token::AssignmentOperator,
            Token::TrueKeyword,
            Token::SemiColon,
            Token::RightCurleyBrace,
            Token::ElseKeyword,
            Token::IfKeyword,
            Token::LeftParenthesis,
            Token::TrueKeyword,
            Token::RightParenthesis,
            Token::LeftCurleyBrace,
            Token::InferKeyword,
            Token::Identifier("c".to_owned()),
            Token::AssignmentOperator,
            Token::TrueKeyword,
            Token::SemiColon,
            Token::RightCurleyBrace,
            Token::ElseKeyword,
            Token::LeftCurleyBrace,
            Token::InferKeyword,
            Token::Identifier("d".to_owned()),
            Token::AssignmentOperator,
            Token::TrueKeyword,
            Token::SemiColon,
            Token::RightCurleyBrace,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        let expected = AstBuilder::default().statement(|statement| {
            statement.if_statement(|if_statement| {
                if_statement
                    .check_expression(|_| true.into())
                    .body(|body| {
                        body.statement(|statement| {
                            statement.var_declaration(|var_declaration| {
                                var_declaration
                                    .infer_type()
                                    .name("a")
                                    .with_assignment(|_| false.into())
                            })
                        })
                        .build()
                    })
                    .else_if(
                        |_| true.into(),
                        |body| {
                            body.statement(|statement| {
                                statement.var_declaration(|var_declaration| {
                                    var_declaration
                                        .infer_type()
                                        .name("b")
                                        .with_assignment(|_| true.into())
                                })
                            })
                            .build()
                        },
                    )
                    .else_if(
                        |_| true.into(),
                        |body| {
                            body.statement(|statement| {
                                statement.var_declaration(|var_declaration| {
                                    var_declaration
                                        .infer_type()
                                        .name("c")
                                        .with_assignment(|_| true.into())
                                })
                            })
                            .build()
                        },
                    )
                    .else_block(|body| {
                        body.statement(|statement| {
                            statement.var_declaration(|var_declaration| {
                                var_declaration
                                    .infer_type()
                                    .name("d")
                                    .with_assignment(|_| true.into())
                            })
                        })
                        .build()
                    })
                    .build()
            })
        });

        assert!(matches!(result, Ok(ast_builder) if ast_builder == expected));
    }

    /// if (true)
    /// {
    ///     my_function();
    ///     if (true)
    ///     {
    ///         my_function();
    ///     }
    ///     my_function();
    /// }
    #[test]
    fn nested_if_statement() {
        let tokens = vec![
            Token::IfKeyword,
            Token::LeftParenthesis,
            Token::TrueKeyword,
            Token::RightParenthesis,
            Token::LeftCurleyBrace,
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
            Token::RightParenthesis,
            Token::SemiColon,
            Token::IfKeyword,
            Token::LeftParenthesis,
            Token::TrueKeyword,
            Token::RightParenthesis,
            Token::LeftCurleyBrace,
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
            Token::RightParenthesis,
            Token::SemiColon,
            Token::RightCurleyBrace,
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
            Token::RightParenthesis,
            Token::SemiColon,
            Token::RightCurleyBrace,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        let function_call_builder = |function_call: FunctionCallBuilder| {
            function_call
                .function_id("my_function")
                .no_parameters()
                .build()
        };
        let expected = AstBuilder::default().statement(|statement| {
            statement.if_statement(|if_statement| {
                if_statement
                    .check_expression(|_| true.into())
                    .body(|body| {
                        body.statement(|statement| {
                            statement.function_call(function_call_builder.clone())
                        })
                        .statement(|statement| {
                            statement.if_statement(|if_statement| {
                                if_statement
                                    .check_expression(|_| true.into())
                                    .body(|body| {
                                        body.statement(|statement| {
                                            statement.function_call(function_call_builder.clone())
                                        })
                                        .build()
                                    })
                                    .build()
                            })
                        })
                        .statement(|statement| {
                            statement.function_call(function_call_builder.clone())
                        })
                        .build()
                    })
                    .build()
            })
        });

        assert!(matches!(result, Ok(ast_builder) if ast_builder == expected));
    }

    /// if (value_a > value_b) {}
    #[test]
    fn if_statement_with_greater_than_operator() {
        let tokens = vec![
            Token::IfKeyword,
            Token::LeftParenthesis,
            Token::Identifier("value_a".to_owned()),
            Token::RightAngle,
            Token::Identifier("value_b".to_owned()),
            Token::RightParenthesis,
            Token::LeftCurleyBrace,
            Token::RightCurleyBrace,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        let expected = AstBuilder::default().statement(|statement| {
            statement.if_statement(|if_statement| {
                if_statement
                    .check_expression(|expression| {
                        expression.operation(|operation| {
                            operation.greater_than(
                                |left| left.variable("value_a"),
                                |right| right.variable("value_b"),
                            )
                        })
                    })
                    .body(|body| body.build())
                    .build()
            })
        });

        assert!(matches!(result, Ok(ast_builder) if ast_builder == expected));
    }
}
