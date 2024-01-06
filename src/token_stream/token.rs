use std::{collections::VecDeque, fmt::Display};

use crate::ast::{
    builders::{
        ast_builder::AstBuilder, expression_builder::ExpressionBuilder,
        statement_builder::StatementBuilder,
        variable_declaration_builder::VariableDeclarationBuilder,
    },
    node::{Expression, Node, Type, VariableDeclarationType},
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
        let mut tokens: VecDeque<Token> = tokens.into();
        while let Some(next_token) = tokens.pop_front() {
            let statement_type = match next_token {
                Token::FunctionKeyword => todo!("function_declaration"),
                Token::Identifier(identifier) => Some(StatementType::FunctionCall(identifier)),
                Token::TypeKeyword(type_) => Some(StatementType::VariableDeclaration(
                    VariableDeclarationType::Type(type_),
                )),
                Token::InferKeyword => Some(StatementType::VariableDeclaration(
                    VariableDeclarationType::Infer,
                )),
                Token::IfKeyword => Some(StatementType::If),
                Token::ReturnKeyword => Some(StatementType::Return),
                _ => {
                    errors.push(TokenStreamError {
                        message: format!("{} is not a valid statement beginning", next_token),
                    });
                    None
                }
            };

            if let Some(statement_type) = statement_type {
                match try_start_statement(statement_type, &mut tokens) {
                    Ok(statement_builder) => {
                        builder = builder.statement(statement_builder);
                    }
                    Err(statement_errors) => {
                        errors.extend(statement_errors);
                    }
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

enum StatementType {
    FunctionCall(String),
    VariableDeclaration(VariableDeclarationType),
    If,
    Return,
}

fn take_while_from_front<T, TPredicate: FnMut(&T) -> bool>(
    items: &mut VecDeque<T>,
    mut predicate: TPredicate,
) -> Vec<T> {
    let mut to_return = Vec::with_capacity(items.len());

    while let Some(item) = items.pop_front() {
        if predicate(&item) {
            to_return.push(item);
        }
    }

    to_return.shrink_to_fit();

    to_return
}

fn try_start_statement(
    statement_type: StatementType,
    mut tokens: &mut VecDeque<Token>,
) -> Result<impl FnOnce(StatementBuilder) -> Node, Vec<TokenStreamError>> {
    match statement_type {
        // variable declaration
        StatementType::VariableDeclaration(var_decl_type) => {
            // variable declaration should end with a semicolon, take all the tokens until the first semicolon
            let mut found_semicolon = false;

            let tokens: VecDeque<_> = take_while_from_front(tokens, |token| {
                found_semicolon = matches!(token, Token::SemiColon);
                !found_semicolon
            })
            .into();

            // we got to the end of the tokens without a semicolon
            if !found_semicolon {
                return Err(vec![TokenStreamError {
                    message: "expected ;".to_owned(),
                }]);
            }

            match try_create_variable_declaration(var_decl_type, tokens) {
                Ok(var_decl_builder) => Ok(|statement_builder: StatementBuilder| {
                    statement_builder.var_declaration(var_decl_builder)
                }),
                Err(errors) => Err(errors),
            }
        }
        StatementType::FunctionCall(identifier) => todo!(),
        StatementType::If => todo!(),
        StatementType::Return => todo!(),
        _ => unreachable!(),
    }
}

fn try_create_variable_declaration(
    var_decl_type: VariableDeclarationType,
    mut tokens: VecDeque<Token>,
) -> Result<impl FnOnce(VariableDeclarationBuilder) -> Node, Vec<TokenStreamError>> {
    let Some(Token::Identifier(name)) = tokens.pop_front() else {
        return Err(vec![TokenStreamError{message: "expected variable identifier".to_owned()}]);
    };

    let cloned_name = name.to_owned();

    if !matches!(tokens.pop_front(), Some(Token::AssignmentOperator)) {
        return Err(vec![TokenStreamError {
            message: "expected assignment operator \"=\"".to_owned(),
        }]);
    }

    let expression_fn = match try_create_expression(&mut tokens) {
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
    mut tokens_iter: &mut VecDeque<Token>,
) -> Result<Box<dyn FnOnce(ExpressionBuilder) -> Expression>, Vec<TokenStreamError>> {
    let mut tokens_iter = Box::new(tokens_iter);
    let expression = match take_expression(&mut tokens_iter) {
        Err(errors) => return Err(errors),
        Ok(value) => value,
    };

    match tokens_iter.pop_front() {
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
    mut tokens_iter: &mut VecDeque<Token>,
) -> Result<Box<dyn FnOnce(ExpressionBuilder) -> Expression>, Vec<TokenStreamError>> {
    let identifier = match tokens_iter.pop_front() {
        Some(Token::FalseKeyword) => {
            return Ok(Box::new(|builder: ExpressionBuilder| {
                builder.value_literal(false.into())
            }))
        }
        Some(Token::TrueKeyword) => {
            return Ok(Box::new(|builder: ExpressionBuilder| {
                builder.value_literal(true.into())
            }))
        }
        Some(Token::UIntValue(value)) => {
            return Ok(Box::new(move |builder: ExpressionBuilder| {
                builder.value_literal(value.into())
            }))
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

    take_identifier_expression(identifier, tokens_iter)
}

/// take an expression from the `tokens_iter` that begins with an identifier. Either a `Token::Variable` or `Token::FunctionCall`
fn take_identifier_expression(
    identifier: String,
    mut tokens: &mut VecDeque<Token>,
) -> Result<Box<dyn FnOnce(ExpressionBuilder) -> Expression>, Vec<TokenStreamError>> {
    match tokens.pop_front() {
        None => Ok(Box::new(move |builder: ExpressionBuilder| {
            builder.variable(identifier.as_str())
        })),
        Some(Token::LeftParenthesis) => match tokens.pop_front() {
            Some(Token::RightParenthesis) => {
                return Ok(Box::new(move |builder: ExpressionBuilder| {
                    builder.function_call(|function_call| {
                        function_call
                            .function_id(&identifier)
                            .no_parameters()
                            .build()
                    })
                }));
            }
            Some(next_token) => {
                tokens.push_front(next_token);
                let param_expression = take_expression(tokens);
                todo!("function call with parameters")
            }
            None => {
                return Err(vec![TokenStreamError {
                    message: "unexpected end of expression".to_owned(),
                }])
            }
        },
        Some(token) => {
            tokens.push_front(token);
            return Ok(Box::new(move |builder: ExpressionBuilder| {
                builder.variable(identifier.as_str())
            }));
        }
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
