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

#[derive(Clone, PartialEq, Debug)]
pub enum Token {
    FunctionKeyword,
    Identifier(String),
    LeftParenthesis,
    RightParenthesis,
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

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

#[derive(Debug)]
pub struct TokenStreamError {
    message: String,
}

impl AstBuilder {
    pub fn from_token_stream(tokens: Vec<Token>) -> Result<Self, Vec<TokenStreamError>> {
        let mut errors = Vec::new();
        let mut builder = AstBuilder::default();
        let mut tokens_iter = tokens.into_iter();
        while let Some(next_token) = dbg!(tokens_iter.next()) {
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

fn try_start_statement(
    first_token: Token,
    mut tokens_iter: &mut impl Iterator<Item = Token>,
) -> Result<impl FnOnce(StatementBuilder) -> Node, Vec<TokenStreamError>> {
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

fn try_create_variable_declaration(
    keyword: Token,
    tokens: Vec<Token>,
) -> Result<impl FnOnce(VariableDeclarationBuilder) -> Node, Vec<TokenStreamError>> {
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

fn try_create_expression(
    mut tokens_iter: impl Iterator<Item = Token>,
) -> Result<Box<dyn FnOnce(ExpressionBuilder) -> Expression>, Vec<TokenStreamError>> {
    let (expression, next_token) = match take_expression(&mut tokens_iter) {
        Err(errors) => return Err(errors),
        Ok(value) => value,
    };

    match dbg!(next_token.or_else(|| tokens_iter.next())) {
        None => return Ok(expression),
        _ => todo!(),
    }

    // if matches!(tokens_iter.next(), Some(Token::TrueKeyword)) {
    //     Ok(Box::new(|builder: ExpressionBuilder| {
    //         builder.value_literal(true.into())
    //     }))
    // } else {
    //     Ok(Box::new(|_| todo!()))
    // }
}

fn take_expression(
    mut tokens_iter: &mut impl Iterator<Item = Token>,
) -> Result<
    (
        Box<dyn FnOnce(ExpressionBuilder) -> Expression>,
        Option<Token>,
    ),
    Vec<TokenStreamError>,
> {
    println!("here");
    let identifier = match dbg!(tokens_iter.next()) {
        Some(Token::FalseKeyword) => {
            return Ok((
                Box::new(|builder: ExpressionBuilder| builder.value_literal(false.into())),
                None,
            ))
        }
        Some(Token::TrueKeyword) => {
            return Ok((
                Box::new(|builder: ExpressionBuilder| builder.value_literal(true.into())),
                None,
            ))
        }
        Some(Token::UIntValue(value)) => {
            return Ok((
                Box::new(move |builder: ExpressionBuilder| builder.value_literal(value.into())),
                None,
            ))
        }
        Some(Token::Identifier(identifier)) => identifier.to_owned(),
        Some(token) => {
            return Err(vec![TokenStreamError {
                message: format!("unexpected token {:?}", token),
            }])
        }
        None => {
            return Err(vec![TokenStreamError {
                message: "unexpected end of tokens".to_owned(),
            }])
        }
    };

    match dbg!(tokens_iter.next()) {
        None => Ok((
            Box::new(move |builder: ExpressionBuilder| builder.variable(identifier.as_str())),
            None,
        )),
        Some(Token::LeftParenthesis) => match tokens_iter.next() {
            Some(Token::RightParenthesis) => {
                return Ok((
                    Box::new(move |builder: ExpressionBuilder| {
                        builder.function_call(|function_call| {
                            function_call
                                .function_id(&identifier)
                                .no_parameters()
                                .build()
                        })
                    }),
                    None,
                ));
            }
            Some(_next_token) => {
                todo!("function call with parameters")
            }
            None => {
                return Err(vec![TokenStreamError {
                    message: "unexpected end of expression".to_owned(),
                }])
            }
        },
        Some(token) => Ok((
            Box::new(move |builder: ExpressionBuilder| builder.variable(identifier.as_str())),
            Some(token),
        )),
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{
        builders::ast_builder::AstBuilder,
        node::{Function, Type},
    };

    use super::Token;

    #[test]
    fn infer_boolean_variable_declaration_from_token_stream() {
        let tokens = vec![
            Token::InferKeyword,
            Token::Identifier("my_var".to_owned()),
            Token::AssignmentOperator,
            Token::TrueKeyword,
            Token::SemiColon,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        let expected = AstBuilder::default().statement(|statement| {
            statement.var_declaration(|var_decl| {
                var_decl
                    .infer_type()
                    .name("my_var")
                    .with_assignment(|_| true.into())
            })
        });

        assert!(matches!(result, Ok(ast_builder) if ast_builder == expected));
    }

    #[test]
    fn infer_variable_declaration_assign_variable_name() {
        let tokens = vec![
            Token::TypeKeyword(Type::Boolean),
            Token::Identifier("my_var".to_owned()),
            Token::AssignmentOperator,
            Token::Identifier("my_other_var".to_owned()),
            Token::SemiColon,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        let expected = AstBuilder::default().statement(|statement| {
            statement.var_declaration(|var_decl| {
                var_decl
                    .name("my_var")
                    .declare_type(Type::Boolean)
                    .with_assignment(|value| value.variable("my_other_var"))
            })
        });

        assert!(matches!(result, Ok(ast_builder) if ast_builder == expected));
    }

    #[test]
    fn infer_variable_declaration_assign_function_call() {
        let tokens = vec![
            Token::TypeKeyword(Type::Boolean),
            Token::Identifier("my_var".to_owned()),
            Token::AssignmentOperator,
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
            Token::RightParenthesis,
            Token::SemiColon,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        let expected = AstBuilder::default().statement(|statement| {
            statement.var_declaration(|var_decl| {
                var_decl
                    .name("my_var")
                    .declare_type(Type::Boolean)
                    .with_assignment(|value| {
                        value.function_call(|function_call| {
                            function_call
                                .function_id("my_function")
                                .no_parameters()
                                .build()
                        })
                    })
            })
        });

        assert!(matches!(dbg!(result), Ok(ast_builder) if ast_builder == expected));
    }
}
