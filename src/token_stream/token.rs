use std::{collections::VecDeque, fmt::Display};

use crate::ast::{
    builders::{ast_builder::AstBuilder, statement_builder::StatementBuilder},
    node::{Node, Type},
};

use super::{function_declaration::build_function_declaration, statement::try_create_statement};

#[derive(Clone, PartialEq, Debug)]
pub enum Token {
    FunctionKeyword,
    Identifier(String),
    LeftParenthesis,
    RightParenthesis,
    FunctionSignitureSplitter, // ->
    UIntValue(u32),
    TypeKeyword(Type),
    TrueKeyword,
    FalseKeyword,
    Comma,
    NotOperator,
    LeftCurleyBrace,
    RightCurleyBrace,
    InferKeyword,
    AssignmentOperator,
    PlusOperator,
    SemiColon,
    IfKeyword,
    ElseKeyword,
    RightAngle, // >
    ReturnKeyword,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

#[derive(Debug)]
pub struct TokenStreamError {
    pub message: String,
}

impl AstBuilder {
    pub fn from_token_stream(tokens: Vec<Token>) -> Result<Self, Vec<TokenStreamError>> {
        let mut errors = Vec::new();
        let mut builder = AstBuilder::default();
        let mut tokens: VecDeque<Token> = tokens.into();
        while let Some(next_token) = tokens.pop_front() {
            match next_token {
                Token::FunctionKeyword => match build_function_declaration(&mut tokens) {
                    Err(function_decl_errors) => errors.extend(function_decl_errors),
                    Ok(function_declaration) => {
                        builder = builder.function_declaration(function_declaration);
                    }
                },
                _ => {
                    tokens.push_front(next_token);

                    match try_create_statement(&mut tokens) {
                        Err(statement_errors) => {
                            errors.extend(statement_errors);
                        }
                        Ok(None) => {
                            errors.push(TokenStreamError {
                                message: format!(
                                    "{} is not a valid statement beginning",
                                    tokens.pop_front().unwrap()
                                ),
                            });
                        }
                        Ok(Some(statement_builder)) => {
                            builder = builder.statement(statement_builder)
                        }
                    }
                }
            };
        }

        if errors.is_empty() {
            Ok(builder)
        } else {
            Err(errors)
        }
    }
}

pub(super) fn take_from_front_while<T, TPredicate: FnMut(&T) -> bool>(
    items: &mut VecDeque<T>,
    mut predicate: TPredicate,
) -> Vec<T> {
    let mut to_return = Vec::with_capacity(items.len());

    while let Some(item) = items.pop_front() {
        if predicate(&item) {
            to_return.push(item);
        } else {
            break;
        }
    }

    to_return.shrink_to_fit();

    to_return
}

pub(super) fn ensure_token(
    tokens: &mut VecDeque<Token>,
    expected: Token,
) -> Result<(), Vec<TokenStreamError>> {
    match tokens.pop_front() {
        None => Err(vec![TokenStreamError {
            message: format!("Expected {}", expected),
        }]),
        Some(token) if token == expected => Ok(()),
        Some(token) => Err(vec![TokenStreamError {
            message: format!("Expected {}, found {}", expected, token),
        }]),
    }
}

pub(super) fn get_block_statements(
    tokens: &mut VecDeque<Token>,
) -> Result<Vec<Box<dyn FnOnce(StatementBuilder) -> Node>>, Vec<TokenStreamError>> {
    let mut require_end_curly_brace = false;
    let mut statements = Vec::new();
    loop {
        match tokens.pop_front() {
            None => {
                return Err(vec![TokenStreamError {
                    message: "expected }".to_owned(),
                }])
            }
            Some(Token::RightCurleyBrace) => {
                break;
            }
            Some(_) if require_end_curly_brace => {
                return Err(vec![TokenStreamError {
                    message: "Expected }".to_owned(),
                }]);
            }
            Some(token) => {
                tokens.push_front(token);

                match try_create_statement(tokens)? {
                    None => {
                        // token is not a valid statement start, next token can only be an end curly brace
                        require_end_curly_brace = true;
                    }
                    Some(statement_builder) => {
                        statements.push(statement_builder);
                    }
                }
            }
        }
    }

    Ok(statements)
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{
            builders::ast_builder::AstBuilder,
            node::{FunctionParameter, Type},
        },
        token_stream::token::Token,
    };

    /// function my_function(boolean param_1, uint param_2) -> uint
    /// {
    ///     print(param_1);
    ///     print(param_2);
    ///     return 1;
    /// }
    /// my_function(true, 10);
    #[test]
    fn function_declaration_and_statements() {
        let tokens = vec![
            Token::FunctionKeyword,
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
            Token::TypeKeyword(Type::Boolean),
            Token::Identifier("param_1".to_owned()),
            Token::Comma,
            Token::TypeKeyword(Type::UInt),
            Token::Identifier("param_2".to_owned()),
            Token::RightParenthesis,
            Token::FunctionSignitureSplitter,
            Token::TypeKeyword(Type::UInt),
            Token::LeftCurleyBrace,
            Token::Identifier("print".to_owned()),
            Token::LeftParenthesis,
            Token::Identifier("param_1".to_owned()),
            Token::RightParenthesis,
            Token::SemiColon,
            Token::Identifier("print".to_owned()),
            Token::LeftParenthesis,
            Token::Identifier("param_2".to_owned()),
            Token::RightParenthesis,
            Token::SemiColon,
            Token::ReturnKeyword,
            Token::UIntValue(1),
            Token::SemiColon,
            Token::RightCurleyBrace,
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
            Token::TrueKeyword,
            Token::Comma,
            Token::UIntValue(10),
            Token::RightParenthesis,
            Token::SemiColon,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        let expected = AstBuilder::default()
            .function_declaration(|function_declaration| {
                function_declaration
                    .name("my_function")
                    .parameters(vec![
                        FunctionParameter::FunctionParameter {
                            param_type: Type::Boolean,
                            param_name: "param_1".to_owned(),
                        },
                        FunctionParameter::FunctionParameter {
                            param_type: Type::UInt,
                            param_name: "param_2".to_owned(),
                        },
                    ])
                    .return_type(Type::UInt)
                    .body(|body| {
                        body.statement(|statement| {
                            statement.function_call(|function_call| {
                                function_call
                                    .function_id("print")
                                    .parameter(|param| param.variable("param_1"))
                                    .build()
                            })
                        })
                        .statement(|statement| {
                            statement.function_call(|function_call| {
                                function_call
                                    .function_id("print")
                                    .parameter(|param| param.variable("param_2"))
                                    .build()
                            })
                        })
                        .statement(|statement| statement.return_value(|_| 1.into()))
                        .build()
                    })
            })
            .statement(|statement| {
                statement.function_call(|function_call| {
                    function_call
                        .function_id("my_function")
                        .parameter(|_| true.into())
                        .parameter(|_| 10.into())
                        .build()
                })
            });

        assert!(matches!(result, Ok(ast_builder) if ast_builder == expected));
    }

    /// function fibonnacci(uint lower, uint higher, uint limit) -> uint
    /// {
    ///     infer next = lower + higher;
    ///     if (next > limit)
    ///     {
    ///         return next;
    ///     }
    ///
    ///     print(next);
    ///
    ///     fibonnacci(higher, next, limit);
    /// }
    ///
    /// print(0);
    /// print(1);
    /// fibonnacci(0, 1, 10000);
    #[test]
    fn fibonacci() {
        let fibonacci_name = "fibonacci".to_owned();
        let lower_name = "lower".to_owned();
        let higher_name = "higher".to_owned();
        let limit_name = "limit".to_owned();
        let next_name = "next".to_owned();
        let tokens = vec![
            // function fibonacci(uint lower, uint higher, uint limit) -> uint {
            Token::FunctionKeyword,
            Token::Identifier(fibonacci_name.clone()),
            Token::LeftParenthesis,
            Token::TypeKeyword(Type::UInt),
            Token::Identifier(lower_name.clone()),
            Token::Comma,
            Token::TypeKeyword(Type::UInt),
            Token::Identifier(higher_name.clone()),
            Token::Comma,
            Token::TypeKeyword(Type::UInt),
            Token::Identifier(limit_name.clone()),
            Token::RightParenthesis,
            Token::FunctionSignitureSplitter,
            Token::TypeKeyword(Type::UInt),
            Token::LeftCurleyBrace,
            // infer next = lower + higher;
            Token::InferKeyword,
            Token::Identifier(next_name.clone()),
            Token::AssignmentOperator,
            Token::Identifier(lower_name.clone()),
            Token::PlusOperator,
            Token::Identifier(higher_name.clone()),
            Token::SemiColon,
            // if (next > limit) { return next; }
            Token::IfKeyword,
            Token::LeftParenthesis,
            Token::Identifier(next_name.clone()),
            Token::RightAngle,
            Token::Identifier(limit_name.clone()),
            Token::RightParenthesis,
            Token::LeftCurleyBrace,
            Token::ReturnKeyword,
            Token::Identifier(next_name.clone()),
            Token::SemiColon,
            Token::RightCurleyBrace,
            // print(next);
            Token::Identifier("print".to_owned()),
            Token::LeftParenthesis,
            Token::Identifier(next_name.to_owned()),
            Token::RightParenthesis,
            Token::SemiColon,
            // fibonacci(higher, next, limit);
            Token::Identifier(fibonacci_name.clone()),
            Token::LeftParenthesis,
            Token::Identifier(higher_name.clone()),
            Token::Comma,
            Token::Identifier(next_name.clone()),
            Token::Comma,
            Token::Identifier(limit_name.clone()),
            Token::RightParenthesis,
            Token::SemiColon,
            // }
            Token::RightCurleyBrace,
            // print(0);
            Token::Identifier("print".to_owned()),
            Token::LeftParenthesis,
            Token::UIntValue(0),
            Token::RightParenthesis,
            Token::SemiColon,
            // print(1);
            Token::Identifier("print".to_owned()),
            Token::LeftParenthesis,
            Token::UIntValue(1),
            Token::RightParenthesis,
            Token::SemiColon,
            // fibonacci(0, 1, 10000);
            Token::Identifier(fibonacci_name.clone()),
            Token::LeftParenthesis,
            Token::UIntValue(0),
            Token::Comma,
            Token::UIntValue(1),
            Token::Comma,
            Token::UIntValue(10000),
            Token::RightParenthesis,
            Token::SemiColon,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        let expected = AstBuilder::default()
            .function_declaration(|function_declaration| {
                function_declaration
                    .name("fibonacci")
                    .parameters(vec![
                        FunctionParameter::FunctionParameter {
                            param_type: Type::UInt,
                            param_name: "lower".to_owned(),
                        },
                        FunctionParameter::FunctionParameter {
                            param_type: Type::UInt,
                            param_name: "higher".to_owned(),
                        },
                        FunctionParameter::FunctionParameter {
                            param_type: Type::UInt,
                            param_name: "limit".to_owned(),
                        },
                    ])
                    .return_type(Type::UInt)
                    .body(|body| {
                        body.statement(|statement| {
                            statement.var_declaration(|var_declaration| {
                                var_declaration
                                    .infer_type()
                                    .name("next")
                                    .with_assignment(|value| {
                                        value.operation(|operation| {
                                            operation.plus(
                                                |left| left.variable("lower"),
                                                |right| right.variable("higher"),
                                            )
                                        })
                                    })
                            })
                        })
                        .statement(|statement| {
                            statement.if_statement(|if_statement| {
                                if_statement
                                    .check_expression(|check| {
                                        check.operation(|operation| {
                                            operation.greater_than(
                                                |left| left.variable("next"),
                                                |right| right.variable("limit"),
                                            )
                                        })
                                    })
                                    .body(|if_body| {
                                        if_body
                                            .statement(|statement| statement.return_value(|value| value.variable("next")))
                                            .build()
                                    })
                                    .build()
                            })
                        })
                        .statement(|statement| {
                            statement.function_call(|function_call| {
                                function_call
                                    .function_id("print")
                                    .parameter(|param| param.variable("next"))
                                    .build()
                            })
                        })
                        .statement(|statement| {
                            statement.function_call(|function_call| {
                                function_call
                                    .function_id("fibonacci")
                                    .parameter(|param| param.variable("higher"))
                                    .parameter(|param| param.variable("next"))
                                    .parameter(|param| param.variable("limit"))
                                    .build()
                            })
                        })
                        .build()
                    })
            })
            .statement(|statement| {
                statement.function_call(|function_call| {
                    function_call
                        .function_id("print")
                        .parameter(|param| param.value_literal(0.into()))
                        .build()
                })
            })
            .statement(|statement| {
                statement.function_call(|function_call| {
                    function_call
                        .function_id("print")
                        .parameter(|param| param.value_literal(1.into()))
                        .build()
                })
            })
            .statement(|statement| {
                statement.function_call(|function_call| {
                    function_call
                        .function_id("fibonacci")
                        .parameter(|param| param.value_literal(0.into()))
                        .parameter(|param| param.value_literal(1.into()))
                        .parameter(|param| param.value_literal(10000.into()))
                        .build()
                })
            });

        assert!(matches!(dbg!(result), Ok(ast_builder) if ast_builder == dbg!(expected)));
    }
}
