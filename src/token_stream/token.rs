use std::{collections::VecDeque, fmt::Display};

use crate::ast::{
    builders::{
        ast_builder::AstBuilder, expression_builder::ExpressionBuilder,
        if_statement_builder::IfStatementBuilder, statement_builder::StatementBuilder,
        variable_declaration_builder::VariableDeclarationBuilder,
    },
    node::{BinaryOperation, Expression, Node, Type, VariableDeclarationType},
};

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
                Token::FunctionKeyword => todo!("function_declaration"),
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

fn try_create_statement(
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

fn take_from_front_while<T, TPredicate: FnMut(&T) -> bool>(
    items: &mut VecDeque<T>,
    mut predicate: TPredicate,
) -> Vec<T> {
    let mut to_return = Vec::with_capacity(items.len());

    while let Some(item) = items.pop_front() {
        if predicate(&item) {
            to_return.push(item);
        }
        else {
            break;
        }
    }

    to_return.shrink_to_fit();

    to_return
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

            match try_create_variable_declaration(var_decl_type, tokens) {
                Ok(var_decl_builder) => Ok(Box::new(|statement_builder: StatementBuilder| {
                    statement_builder.var_declaration(var_decl_builder)
                })),
                Err(errors) => Err(errors),
            }
        }
        StatementType::FunctionCall(_identifier) => todo!(),
        StatementType::If => match try_create_if_statement(tokens) {
            Err(err) => Err(err),
            Ok(if_statement_builder) => Ok(Box::new(|statement_builder: StatementBuilder| {
                statement_builder.if_statement(if_statement_builder)
            })),
        },
        StatementType::Return => todo!(),
    }
}

fn try_create_if_statement(
    tokens: &mut VecDeque<Token>,
) -> Result<impl FnOnce(IfStatementBuilder) -> Node, Vec<TokenStreamError>> {
    match tokens.pop_front() {
        None => {
            return Err(vec![TokenStreamError {
                message: "unexpected end of if statement".to_owned(),
            }])
        }
        Some(Token::LeftParenthesis) => {}
        Some(token) => {
            return Err(vec![TokenStreamError {
                message: format!("unexpected token {:?}", token),
            }])
        }
    };

    let expression = match take_expression(tokens) {
        Err(errors) => return Err(errors),
        Ok(expression) => expression,
    };

    match tokens.pop_front() {
        None => {
            return Err(vec![TokenStreamError {
                message: "unexpected end of if statement".to_owned(),
            }])
        }
        Some(Token::RightParenthesis) => {}
        Some(token) => {
            return Err(vec![TokenStreamError {
                message: format!("unexpected token {:?}", token),
            }])
        }
    }

match tokens.pop_front() {
        None => {
            return Err(vec![TokenStreamError {
                message: "unexpected end of if statement".to_owned(),
            }])
        }
        Some(Token::LeftCurleyBrace) => {}
        Some(token) => {
            return Err(vec![TokenStreamError {
                message: format!("unexpected token {:?}", token),
            }])
        }
    }

    let mut statements = Vec::new();
    let mut require_end_curly_brace = false;

    loop {
        match tokens.pop_front() {
            None => {
                return Err(vec![TokenStreamError {
                    message: "unexpected end of if statement".to_owned(),
                }])
            }
            Some(Token::RightCurleyBrace) => {
                return Ok(move |if_statement_builder: IfStatementBuilder| {
                    if_statement_builder
                        .check_expression(expression)
                        .body(|mut body| {
                            for statement in statements.into_iter() {
                                body = body.statement(statement);
                            }

                            body.build()
                        })
                        .build()
                });
            }
            Some(_) if require_end_curly_brace => {
                return Err(vec![TokenStreamError {
                    message: "Expected }".to_owned(),
                }]);
            }
            Some(token) => {
                tokens.push_front(token);

                match try_create_statement(tokens) {
                    Err(statement_errors) => return Err(statement_errors),
                    Ok(None) => {
                        // token is not a valid statement start, next token can only be an end curly brace
                        require_end_curly_brace = true;
                    }
                    Ok(Some(statement_builder)) => {
                        statements.push(statement_builder);
                    }
                }
            }
        }
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

    let expression_fn = match take_expression(&mut tokens) {
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

fn take_expression(
    tokens: &mut VecDeque<Token>,
) -> Result<Box<dyn FnOnce(ExpressionBuilder) -> Expression>, Vec<TokenStreamError>> {
    match tokens.pop_front() {
        Some(Token::FalseKeyword) => take_value_expression(
            Box::new(|builder: ExpressionBuilder| builder.value_literal(false.into())),
            tokens,
        ),
        Some(Token::TrueKeyword) => take_value_expression(
            Box::new(|builder: ExpressionBuilder| builder.value_literal(true.into())),
            tokens,
        ),
        Some(Token::UIntValue(value)) => take_value_expression(
            Box::new(move |builder: ExpressionBuilder| builder.value_literal(value.into())),
            tokens,
        ),
        Some(Token::Identifier(identifier)) => take_identifier_expression(identifier, tokens),
        Some(Token::NotOperator) => match take_expression(tokens) {
            Err(errors) => Err(errors),
            Ok(value_expr) => Ok(Box::new(move |builder: ExpressionBuilder| {
                builder.operation(|operation| operation.not(value_expr))
            })),
        },
        Some(token) => Err(vec![TokenStreamError {
            message: format!("unexpected token {:?}", token),
        }]),
        None => Err(vec![TokenStreamError {
            message: "unexpected end of tokens".to_owned(),
        }]),
    }
}

fn take_value_expression(
    value_expression: Box<dyn FnOnce(ExpressionBuilder) -> Expression>,
    tokens: &mut VecDeque<Token>,
) -> Result<Box<dyn FnOnce(ExpressionBuilder) -> Expression>, Vec<TokenStreamError>> {
    match tokens.pop_front() {
        None => Ok(value_expression),
        Some(Token::PlusOperator) => {
            take_binary_operation_expression(BinaryOperation::Plus, value_expression, tokens)
        }
        Some(Token::RightAngle) => {
            take_binary_operation_expression(BinaryOperation::GreaterThan, value_expression, tokens)
        }
        Some(token) => {
            tokens.push_front(token);
            Ok(value_expression)
        }
    }
}

fn take_binary_operation_expression(
    operation: BinaryOperation,
    left_expression: Box<dyn FnOnce(ExpressionBuilder) -> Expression>,
    tokens: &mut VecDeque<Token>,
) -> Result<Box<dyn FnOnce(ExpressionBuilder) -> Expression>, Vec<TokenStreamError>> {
    match take_expression(tokens) {
        Err(errors) => Err(errors),
        Ok(right_expression) => Ok(Box::new(move |expression_builder: ExpressionBuilder| {
            expression_builder.operation(|operation_builder| match operation {
                BinaryOperation::GreaterThan => {
                    operation_builder.greater_than(left_expression, right_expression)
                }
                BinaryOperation::Plus => operation_builder.plus(left_expression, right_expression),
            })
        })),
    }
}

/// take an expression from the `tokens` that begins with an identifier. Either a `Token::Variable` or `Token::FunctionCall`
fn take_identifier_expression(
    identifier: String,
    tokens: &mut VecDeque<Token>,
) -> Result<Box<dyn FnOnce(ExpressionBuilder) -> Expression>, Vec<TokenStreamError>> {
    match tokens.pop_front() {
        None => Ok(Box::new(move |builder: ExpressionBuilder| {
            builder.variable(identifier.as_str())
        })),
        Some(Token::LeftParenthesis) => take_function_call(tokens, identifier),
        Some(token) => {
            tokens.push_front(token);
            return Ok(Box::new(move |builder: ExpressionBuilder| {
                builder.variable(identifier.as_str())
            }));
        }
    }
}

fn take_function_call(
    tokens: &mut VecDeque<Token>,
    identifier: String,
) -> Result<Box<dyn FnOnce(ExpressionBuilder) -> Expression>, Vec<TokenStreamError>> {
    let mut params = VecDeque::new();

    let mut found_comma = false;

    loop {
        match tokens.pop_front() {
            None => {
                return Err(vec![TokenStreamError {
                    message: "unexpected end of function call".to_owned(),
                }])
            }
            Some(Token::RightParenthesis) => {
                return Ok(Box::new(move |expression_builder| {
                    expression_builder.function_call(|mut function_call| {
                        function_call = function_call.function_id(&identifier);
                        if params.is_empty() {
                            function_call = function_call.no_parameters();
                        } else {
                            while let Some(param) = params.pop_front() {
                                function_call = function_call.parameter(param);
                            }
                        }
                        function_call.build()
                    })
                }))
            }
            Some(Token::Comma) => {
                if params.is_empty() {
                    return Err(vec![TokenStreamError {
                        message: "unexpected ,".to_owned(),
                    }]);
                }
                found_comma = true;
            }
            Some(token) => {
                // not a comma, if we haven't seen a comma since the last parameter, then err
                if params.len() > 0 && !found_comma {
                    return Err(vec![TokenStreamError {
                        message: "Require comma separating parameters".to_owned(),
                    }]);
                }

                // reset found_comma
                found_comma = false;

                tokens.push_front(token);
                match take_expression(tokens) {
                    Err(errors) => {
                        return Err(errors);
                    }
                    Ok(param) => {
                        params.push_back(param);
                    }
                }
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{builders::ast_builder::AstBuilder, node::Type};

    use super::Token;

    /// infer my_var = true;
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

    /// boolean my_var = my_other_var;
    #[test]
    fn type_decl_variable_declaration_assign_variable_name() {
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

    /// boolean my_var = my_function();
    #[test]
    fn type_decl_variable_declaration_assign_function_call() {
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

        assert!(matches!(result, Ok(ast_builder) if ast_builder == expected));
    }

    /// boolean my_var = my_function(true);
    #[test]
    fn type_declare_variable_declaration_assign_function_call_with_single_paremeter() {
        let tokens = vec![
            Token::TypeKeyword(Type::Boolean),
            Token::Identifier("my_var".to_owned()),
            Token::AssignmentOperator,
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
            Token::TrueKeyword,
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
                                .parameter(|value| value.value_literal(true.into()))
                                .build()
                        })
                    })
            })
        });

        assert!(matches!(result, Ok(ast_builder) if ast_builder == expected));
    }

    /// boolean my_var = my_function(true, second_function());
    #[test]
    fn variable_declaration_assign_function_call_with_multiple_parameters_function_call_no_parameters(
    ) {
        let tokens = vec![
            Token::TypeKeyword(Type::Boolean),
            Token::Identifier("my_var".to_owned()),
            Token::AssignmentOperator,
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
            Token::TrueKeyword,
            Token::Comma,
            Token::Identifier("second_function".to_owned()),
            Token::LeftParenthesis,
            Token::RightParenthesis,
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
                                .parameter(|value| value.value_literal(true.into()))
                                .parameter(|param| {
                                    param.function_call(|function_call| {
                                        function_call
                                            .function_id("second_function")
                                            .no_parameters()
                                            .build()
                                    })
                                })
                                .build()
                        })
                    })
            })
        });

        assert!(matches!(result, Ok(ast_builder) if ast_builder == expected));
    }

    /// boolean my_var = my_function(true, second_function(true));
    #[test]
    fn variable_declaration_assign_function_call_with_multiple_parameters_function_call_single_parameter(
    ) {
        let tokens = vec![
            Token::TypeKeyword(Type::Boolean),
            Token::Identifier("my_var".to_owned()),
            Token::AssignmentOperator,
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
            Token::TrueKeyword,
            Token::Comma,
            Token::Identifier("second_function".to_owned()),
            Token::LeftParenthesis,
            Token::TrueKeyword,
            Token::RightParenthesis,
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
                                .parameter(|value| value.value_literal(true.into()))
                                .parameter(|param| {
                                    param.function_call(|function_call| {
                                        function_call
                                            .function_id("second_function")
                                            .parameter(|param| param.value_literal(true.into()))
                                            .build()
                                    })
                                })
                                .build()
                        })
                    })
            })
        });

        assert!(matches!(result, Ok(ast_builder) if ast_builder == expected));
    }

    /// boolean my_var = my_function(true, false, true);
    #[test]
    fn variable_declaration_assign_function_call_with_three_parameters() {
        let tokens = vec![
            Token::TypeKeyword(Type::Boolean),
            Token::Identifier("my_var".to_owned()),
            Token::AssignmentOperator,
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
            Token::TrueKeyword,
            Token::Comma,
            Token::FalseKeyword,
            Token::Comma,
            Token::TrueKeyword,
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
                                .parameter(|value| value.value_literal(true.into()))
                                .parameter(|value| value.value_literal(false.into()))
                                .parameter(|value| value.value_literal(true.into()))
                                .build()
                        })
                    })
            })
        });

        assert!(matches!(result, Ok(ast_builder) if ast_builder == expected));
    }

    /// boolean my_var = my_function(true false true);
    #[test]
    fn function_call_requires_comma_between_params() {
        let tokens = vec![
            Token::TypeKeyword(Type::Boolean),
            Token::Identifier("my_var".to_owned()),
            Token::AssignmentOperator,
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
            Token::TrueKeyword,
            Token::FalseKeyword,
            Token::TrueKeyword,
            Token::RightParenthesis,
            Token::SemiColon,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        assert!(matches!(result, Err(_)));
    }

    /// infer my_var = my_function(,);
    #[test]
    fn function_call_fails_when_no_expression() {
        let tokens = vec![
            Token::InferKeyword,
            Token::Identifier("my_var".to_owned()),
            Token::AssignmentOperator,
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
            Token::Comma,
            Token::RightParenthesis,
            Token::SemiColon,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        assert!(matches!(result, Err(_)));
    }

    /// infer my_var = my_function(10+12);
    #[test]
    fn function_call_plus_operation() {
        let tokens = vec![
            Token::InferKeyword,
            Token::Identifier("my_var".to_owned()),
            Token::AssignmentOperator,
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
            Token::UIntValue(10),
            Token::PlusOperator,
            Token::UIntValue(12),
            Token::RightParenthesis,
            Token::SemiColon,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        let expected = AstBuilder::default().statement(|statement| {
            statement.var_declaration(|var_decl| {
                var_decl
                    .infer_type()
                    .name("my_var")
                    .with_assignment(|assignment| {
                        assignment.function_call(|function_call| {
                            function_call
                                .function_id("my_function")
                                .parameter(|parameter| {
                                    parameter.operation(|operation| {
                                        operation.plus(|_| 10.into(), |_| 12.into())
                                    })
                                })
                                .build()
                        })
                    })
            })
        });

        assert!(matches!(result, Ok(ast_builder) if ast_builder == expected));
    }

    /// infer my_var = my_function(!true);
    #[test]
    fn function_call_not_operation() {
        let tokens = vec![
            Token::InferKeyword,
            Token::Identifier("my_var".to_owned()),
            Token::AssignmentOperator,
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
            Token::NotOperator,
            Token::TrueKeyword,
            Token::RightParenthesis,
            Token::SemiColon,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        let expected = AstBuilder::default().statement(|statement| {
            statement.var_declaration(|var_decl| {
                var_decl
                    .infer_type()
                    .name("my_var")
                    .with_assignment(|assignment| {
                        assignment.function_call(|function_call| {
                            function_call
                                .function_id("my_function")
                                .parameter(|param| {
                                    param.operation(|operation| operation.not(|_| true.into()))
                                })
                                .build()
                        })
                    })
            })
        });

        assert!(matches!(result, Ok(ast_builder) if ast_builder == expected))
    }

    /// infer my_var = my_function(10>12);
    #[test]
    fn function_call_greater_than_operation() {
        let tokens = vec![
            Token::InferKeyword,
            Token::Identifier("my_var".to_owned()),
            Token::AssignmentOperator,
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
            Token::UIntValue(10),
            Token::RightAngle,
            Token::UIntValue(12),
            Token::RightParenthesis,
            Token::SemiColon,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        let expected = AstBuilder::default().statement(|statement| {
            statement.var_declaration(|var_decl| {
                var_decl
                    .infer_type()
                    .name("my_var")
                    .with_assignment(|assignment| {
                        assignment.function_call(|function_call| {
                            function_call
                                .function_id("my_function")
                                .parameter(|parameter| {
                                    parameter.operation(|operation| {
                                        operation.greater_than(|_| 10.into(), |_| 12.into())
                                    })
                                })
                                .build()
                        })
                    })
            })
        });

        assert!(matches!(result, Ok(ast_builder) if ast_builder == expected));
    }

    /// infer my_var = 10+11+12;
    #[test]
    fn multiple_plus_operations() {
        let tokens = vec![
            Token::InferKeyword,
            Token::Identifier("my_var".to_owned()),
            Token::AssignmentOperator,
            Token::UIntValue(10),
            Token::PlusOperator,
            Token::UIntValue(11),
            Token::PlusOperator,
            Token::UIntValue(12),
            Token::SemiColon,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        // todo: evaluate left to right
        let expected = AstBuilder::default().statement(|statement| {
            statement.var_declaration(|var_declaration| {
                var_declaration
                    .infer_type()
                    .name("my_var")
                    .with_assignment(|value| {
                        value.operation(|operation| {
                            operation.plus(
                                |_| 10.into(),
                                |right| {
                                    right.operation(|operation| {
                                        operation.plus(|_| 11.into(), |_| 12.into())
                                    })
                                },
                            )
                        })
                    })
            })
        });

        assert!(matches!(result, Ok(ast_builder) if ast_builder == expected));
    }

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
}
