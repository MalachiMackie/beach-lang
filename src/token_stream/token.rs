use std::{
    collections::{hash_map::RandomState, HashSet},
    fmt::Display,
    hash,
};

use crate::ast::{
    builders::{
        ast_builder::AstBuilder, expression_builder::ExpressionBuilder,
        statement_builder::StatementBuilder,
        variable_declaration_builder::VariableDeclarationBuilder,
    },
    node::{Expression, Node, Type, Value, VariableDeclarationType},
};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Token<'code> {
    FunctionKeyword,
    Identifier(&'code str),
    LeftParenthesis,
    RightParanthesis,
    UIntValue(u32),
    TypeKeyword(Type),
    TrueKeyword,
    FalseKeyword,
    Comma,
    RightArrow,
    LeftCurleyBrace,
    RightCurleyBrace,
    InferKeyword,
    AssignmentOperator,
    PlusOperator,
    SemiColon,
    IfKeyword,
    GreaterThanOperator,
    ReturnKeyword,
}

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

#[derive(Debug)]
pub struct TokenStreamError {
    message: String,
}

impl AstBuilder {
    pub fn from_token_stream(tokens: &[Token]) -> Result<Self, Vec<TokenStreamError>> {
        let mut errors = Vec::new();
        let mut builder = AstBuilder::default();
        let mut tokens_iter = tokens.iter().copied();
        while let Some(next_token) = tokens_iter.next() {
            match next_token {
                Token::FunctionKeyword => todo!("function_declaration"),
                Token::Identifier(_)
                | Token::UIntValue(_)
                | Token::TypeKeyword(_)
                | Token::TrueKeyword
                | Token::FalseKeyword
                | Token::InferKeyword
                | Token::IfKeyword
                | Token::ReturnKeyword => match try_start_statement(next_token, &mut tokens_iter) {
                    Ok(statement_builder) => {
                        builder = builder.statement(statement_builder);
                    }
                    Err(statement_errors) => {
                        errors.extend(statement_errors);
                    }
                },
                _ => {
                    errors.push(TokenStreamError {
                        message: format!("{} is not a valid statement beginning", next_token),
                    });
                }
            }
        }

        if errors.is_empty() {
            Ok(builder)
        } else {
            Err(errors)
        }
    }
}

fn try_start_statement<'a>(
    first_token: Token<'a>,
    mut tokens_iter: &mut impl Iterator<Item = Token<'a>>,
) -> Result<impl FnOnce(StatementBuilder) -> Node + 'a, Vec<TokenStreamError>> {
    match first_token {
        // variable declaration
        Token::TypeKeyword(_) | Token::InferKeyword => {
            // variable declaration should end with a semicolon, take all the tokens until the first semicolon
            let mut found_semicolon = false;
            let tokens: Vec<_> = tokens_iter
                .by_ref()
                .take_while(|token| {
                    found_semicolon = matches!(token, Token::SemiColon);
                    !found_semicolon
                })
                .collect();

            // we got to the end of the tokens without a semicolon
            if !found_semicolon {
                return Err(vec![TokenStreamError {
                    message: "expected ;".to_owned(),
                }]);
            }

            match try_create_variable_declaration(first_token, tokens) {
                Ok(var_decl_builder) => Ok(|statement_builder: StatementBuilder| {
                    statement_builder.var_declaration(var_decl_builder)
                }),
                Err(errors) => Err(errors),
            }
        }
        // perform operation on a value
        Token::UIntValue(_) | Token::TrueKeyword | Token::FalseKeyword | Token::Identifier(_) => {
            todo!()
        }
        Token::IfKeyword => todo!(),
        Token::ReturnKeyword => todo!(),
        _ => unreachable!(),
    }
}

fn try_create_variable_declaration<'a>(
    keyword: Token<'a>,
    tokens: Vec<Token<'a>>,
) -> Result<impl FnOnce(VariableDeclarationBuilder) -> Node + 'a, Vec<TokenStreamError>> {
    let var_decl_type = match keyword {
        Token::InferKeyword => VariableDeclarationType::Infer,
        Token::TypeKeyword(found_type) => VariableDeclarationType::Type(found_type),
        _ => unreachable!(),
    };

    let mut tokens_iter = tokens.into_iter();

    let Some(Token::Identifier(name)) = tokens_iter.next() else {
        return Err(vec![TokenStreamError{message: "expected variable identifier".to_owned()}]);
    };

    let cloned_name = name.to_owned();

    if !matches!(tokens_iter.next(), Some(Token::AssignmentOperator)) {
        return Err(vec![TokenStreamError {
            message: "expected assignment operator \"=\"".to_owned(),
        }]);
    }

    let expression_fn = match try_create_expression(tokens_iter) {
        Ok(expression_fn) => expression_fn,
        Err(errors) => return Err(errors),
    };

    Ok(move |mut var_decl_builder: VariableDeclarationBuilder| {
        match var_decl_type {
            VariableDeclarationType::Infer => {
                var_decl_builder = var_decl_builder.infer_type();
            }
            VariableDeclarationType::Type(var_type) => {
                var_decl_builder = var_decl_builder.declare_type(var_type);
            }
        }
        var_decl_builder = var_decl_builder.name(&cloned_name);
        var_decl_builder.with_assignment(expression_fn)
    })
}

#[derive(PartialEq, Eq, Hash)]
enum ExpressionType {
    ValueLiteral,
    FunctionCall,
    Operation,
    VariableAccess,
}

fn try_create_expression<'a>(
    mut tokens_iter: impl Iterator<Item = Token<'a>>,
) -> Result<Box<dyn FnOnce(ExpressionBuilder) -> Expression>, Vec<TokenStreamError>> {
    if matches!(tokens_iter.next(), Some(Token::TrueKeyword)) {
        Ok(Box::new(|builder: ExpressionBuilder| {
            builder.value_literal(true.into())
        }))
    } else {
        Ok(Box::new(|_| todo!()))
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::builders::ast_builder::AstBuilder;

    use super::Token;

    #[test]
    fn infer_boolean_variable_declaration_from_token_stream() {
        let tokens = [
            Token::InferKeyword,
            Token::Identifier("my_var"),
            Token::AssignmentOperator,
            Token::TrueKeyword,
            Token::SemiColon,
        ];

        let result = AstBuilder::from_token_stream(&tokens);

        let expected = AstBuilder::default().statement(|statement| {
            statement.var_declaration(|var_decl| {
                var_decl
                    .infer_type()
                    .name("my_var")
                    .with_assignment(|_| true.into())
            })
        });

        assert!(matches!(dbg!(result), Ok(ast_builder) if ast_builder == expected));
    }
}
