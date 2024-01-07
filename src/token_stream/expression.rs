use std::{collections::VecDeque, process::id};

use crate::ast::{
    builders::expression_builder::{self, ExpressionBuilder},
    node::{BinaryOperation, Expression, Operation},
};

use super::{
    function_call::take_function_call,
    token::{Token, TokenStreamError},
};

pub(super) fn take_expression(
    tokens: &mut VecDeque<Token>,
) -> Result<Box<dyn FnOnce(ExpressionBuilder) -> Expression>, Vec<TokenStreamError>> {
    let mut expression = None;
    loop {
        match tokens.pop_front() {
            None => {
                if let Some(expression) = expression {
                    return Ok(expression);
                }
                return Err(vec![TokenStreamError {
                    message: "expected expression".to_owned(),
                }]);
            }
            Some(Token::FalseKeyword)
            | Some(Token::TrueKeyword)
            | Some(Token::UIntValue(_))
            | Some(Token::Identifier(_))
            | Some(Token::NotOperator)
                if expression.is_some() =>
            {
                return Err(vec![TokenStreamError {
                    message: "expected +, > or (".to_owned(),
                }]);
            }
            Some(Token::FalseKeyword) => {
                expression = Some(take_value_expression(
                    Box::new(|builder: ExpressionBuilder| builder.value_literal(false.into())),
                    tokens,
                )?);
            }
            Some(Token::TrueKeyword) => {
                expression = Some(take_value_expression(
                    Box::new(|builder: ExpressionBuilder| builder.value_literal(true.into())),
                    tokens,
                )?)
            }
            Some(Token::UIntValue(value)) => {
                expression = Some(take_value_expression(
                    Box::new(move |builder: ExpressionBuilder| builder.value_literal(value.into())),
                    tokens,
                )?)
            }
            Some(Token::Identifier(identifier)) => {
                expression = Some(take_identifier_expression(identifier, tokens)?)
            }
            Some(Token::NotOperator) => {
                let value_expr = take_expression(tokens)?;
                expression = Some(Box::new(move |builder: ExpressionBuilder| {
                    builder.operation(|operation| operation.not(value_expr))
                }));
            }
            Some(Token::PlusOperator) => {
                if let Some(some_expression) = expression {
                    expression = Some(take_binary_operation_expression(
                        BinaryOperation::Plus,
                        some_expression,
                        tokens,
                    )?);
                } else {
                    return Err(vec![TokenStreamError {
                        message: "Expected expression".to_owned(),
                    }]);
                }
            }
            Some(Token::RightAngle) => {
                if let Some(some_expression) = expression {
                    expression = Some(take_binary_operation_expression(
                        BinaryOperation::GreaterThan,
                        some_expression,
                        tokens,
                    )?);
                } else {
                    return Err(vec![TokenStreamError {
                        message: "Expected expression".to_owned(),
                    }]);
                }
            }
            Some(token) => {
                if let Some(expression) = expression {
                    tokens.push_front(token);
                    return Ok(expression);
                }
                return Err(vec![TokenStreamError {
                    message: format!("unexpected token {:?}", token),
                }]);
            }
        }
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
    let right_expression = take_expression(tokens)?;

    Ok(Box::new(move |expression_builder: ExpressionBuilder| {
        expression_builder.operation(|operation_builder| match operation {
            BinaryOperation::GreaterThan => {
                operation_builder.greater_than(left_expression, right_expression)
            }
            BinaryOperation::Plus => operation_builder.plus(left_expression, right_expression),
        })
    }))
}

/// take an expression from the `tokens` that begins with an identifier. Either a `Token::Variable` or `Token::FunctionCall`
fn take_identifier_expression(
    identifier: String,
    tokens: &mut VecDeque<Token>,
) -> Result<Box<dyn FnOnce(ExpressionBuilder) -> Expression>, Vec<TokenStreamError>> {
    match tokens.pop_front() {
        None => Ok(Box::new(move |expression_builder| {
            expression_builder.variable(&identifier)
        })),
        Some(Token::LeftParenthesis) => {
            tokens.push_front(Token::LeftParenthesis);
            take_function_call_expression(tokens, identifier)
        }
        Some(token) => {
            tokens.push_front(token);
            Ok(Box::new(move |expression_builder| {
                expression_builder.variable(&identifier)
            }))
        }
    }
    // let cloned_identifier = identifier.clone();
    // let mut expression: Box<dyn FnOnce(ExpressionBuilder) -> Expression> =
    //     Box::new(move |builder: ExpressionBuilder| builder.variable(cloned_identifier.as_str()));
    // let mut reassigned = false;
    // loop {
    //     match tokens.pop_front() {
    //         None => return Ok(expression),
    //         Some(Token::RightAngle) => {
    //             expression = take_binary_operation_expression(
    //                 BinaryOperation::GreaterThan,
    //                 expression,
    //                 tokens,
    //             )?;
    //             reassigned = true;
    //         }
    //         Some(Token::PlusOperator) => {
    //             expression =
    //                 take_binary_operation_expression(BinaryOperation::Plus, expression, tokens)?;
    //             reassigned = true;
    //         }
    //         Some(Token::LeftParenthesis) if !reassigned => {
    //             tokens.push_front(Token::LeftParenthesis);
    //             expression = take_function_call_expression(tokens, identifier.clone())?;
    //         }
    //         Some(Token::LeftParenthesis) => {
    //             // todo: change function call to actually take an expression for the function, so we can return functions
    //             return Err(vec![TokenStreamError {
    //                 message: "cannot call this thing".to_owned(),
    //             }]);
    //         }
    //         Some(token) => {
    //             tokens.push_front(token);
    //             return Ok(expression);
    //         }
    //     }
    // }
}

fn take_function_call_expression(
    tokens: &mut VecDeque<Token>,
    identifier: String,
) -> Result<Box<dyn FnOnce(ExpressionBuilder) -> Expression>, Vec<TokenStreamError>> {
    let function_call = take_function_call(identifier, tokens)?;

    Ok(Box::new(|expression_builder: ExpressionBuilder| {
        expression_builder.function_call(function_call)
    }))
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{builders::ast_builder::AstBuilder, node::Type},
        token_stream::token::Token,
    };

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

    #[test]
    fn greater_than_function_calls() {
        let tokens = vec![
            Token::InferKeyword,
            Token::Identifier("my_var".to_owned()),
            Token::AssignmentOperator,
            Token::Identifier("function_1".to_owned()),
            Token::LeftParenthesis,
            Token::RightParenthesis,
            Token::RightAngle,
            Token::Identifier("function_2".to_owned()),
            Token::LeftParenthesis,
            Token::RightParenthesis,
            Token::SemiColon,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        let expected = AstBuilder::default().statement(|statement| {
            statement.var_declaration(|var_declaration| {
                var_declaration
                    .name("my_var")
                    .infer_type()
                    .with_assignment(|value| {
                        value.operation(|operation| {
                            operation.greater_than(
                                |left| {
                                    left.function_call(|function_call| {
                                        function_call.function_id("function_1").no_parameters().build()
                                    })
                                },
                                |right| right.function_call(|function_call| function_call.function_id("function_2").no_parameters().build()),
                            )
                        })
                    })
            })
        });

        assert!(matches!(result, Ok(ast_builder) if ast_builder == expected));
    }
}
