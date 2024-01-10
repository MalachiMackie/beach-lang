mod ast;
mod evaluation;
mod token_stream;
mod type_checking;
mod parsing;

use ast::{
    builders::ast_builder::AstBuilder,
    node::{Ast, FunctionParameter},
};
use token_stream::token::{Token, TokenStreamError};

use crate::ast::node::Type;

fn main() {
    let ast = match fibonacci_tokens() {
        Err(errors) => {
            for error in errors {
                println!("{}", error.message);
            }

            return;
        },
        Ok(ast) => ast
    };

    if let Err(errors) = ast.type_check() {
        for error in errors {
            println!("{}", error.message);
        }

        return;
    }

    ast.evaluate();
}

fn fibonacci_tokens() -> Result<Ast, Vec<TokenStreamError>> {
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

    AstBuilder::from_token_stream(tokens).map(|ast| ast.build())
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
fn fibonacci_ast_builder(ast_builder: AstBuilder) -> Ast {
    ast_builder
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
                    }).build()
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
        })
        .build()
}
