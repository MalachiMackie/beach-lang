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
    RightArrow,
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
