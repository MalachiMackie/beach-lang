use std::collections::VecDeque;

use crate::ast::{
    builders::statement_builder::StatementBuilder,
    node::{Node, VariableDeclarationType},
};

use super::{
    if_statement::try_create_if_statement,
    token::{take_from_front_while, Token, TokenStreamError},
    variable_declaration::try_create_variable_declaration,
};

pub(super) fn try_create_statement(
    tokens: &mut VecDeque<Token>,
) -> Result<Option<Box<dyn FnOnce(StatementBuilder) -> Node>>, Vec<TokenStreamError>> {
    let token = tokens.pop_front();
    let statement_type = match token {
        None => {
            return Err(vec![TokenStreamError {
                message: "Unexpected end of tokens".to_owned(),
            }]);
        }
        Some(Token::Identifier(identifier)) => StatementType::FunctionCall(identifier),
        Some(Token::TypeKeyword(type_)) => {
            StatementType::VariableDeclaration(VariableDeclarationType::Type(type_))
        }
        Some(Token::InferKeyword) => {
            StatementType::VariableDeclaration(VariableDeclarationType::Infer)
        }
        Some(Token::IfKeyword) => StatementType::If,
        Some(Token::ReturnKeyword) => StatementType::Return,
        Some(token) => {
            tokens.push_front(token);
            return Ok(None);
        }
    };

    try_start_statement(statement_type, tokens).map(|x| Some(x))
}

enum StatementType {
    FunctionCall(String),
    VariableDeclaration(VariableDeclarationType),
    If,
    Return,
}

fn try_start_statement(
    statement_type: StatementType,
    tokens: &mut VecDeque<Token>,
) -> Result<Box<dyn FnOnce(StatementBuilder) -> Node>, Vec<TokenStreamError>> {
    match statement_type {
        // variable declaration
        StatementType::VariableDeclaration(var_decl_type) => {
            // variable declaration should end with a semicolon, take all the tokens until the first semicolon
            let mut found_semicolon = false;

            let tokens: VecDeque<_> = take_from_front_while(tokens, |token| {
                if !found_semicolon {
                    found_semicolon = matches!(token, Token::SemiColon);
                }
                !found_semicolon
            })
            .into();

            // we got to the end of the tokens without a semicolon
            if !found_semicolon {
                return Err(vec![TokenStreamError {
                    message: "expected ;".to_owned(),
                }]);
            }

            let var_decl_builder = try_create_variable_declaration(var_decl_type, tokens)?;
            Ok(Box::new(|statement_builder: StatementBuilder| {
                statement_builder.var_declaration(var_decl_builder)
            }))
        }
        StatementType::FunctionCall(_identifier) => todo!(),
        StatementType::If => {
            let if_statement_builder = try_create_if_statement(tokens)?;
            Ok(Box::new(|statement_builder: StatementBuilder| {
                statement_builder.if_statement(if_statement_builder)
            }))
        }
        StatementType::Return => todo!(),
    }
}
