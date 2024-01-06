use std::collections::VecDeque;

use crate::ast::{
    builders::function_declaration_builder::FunctionDeclarationBuilder,
    node::{FunctionDeclaration, FunctionParameter},
};

use super::token::{ensure_token, get_block_statements, Token, TokenStreamError};

pub(super) fn build_function_declaration(
    tokens: &mut VecDeque<Token>,
) -> Result<Box<dyn FnOnce(FunctionDeclarationBuilder) -> FunctionDeclaration>, Vec<TokenStreamError>>
{
    let function_name = match tokens.pop_front() {
        None => {
            return Err(vec![TokenStreamError {
                message: "expected function name".to_owned(),
            }])
        }
        Some(Token::Identifier(function_name)) => function_name,
        Some(token) => {
            return Err(vec![TokenStreamError {
                message: format!("expected function_name. found {}", token),
            }])
        }
    };

    ensure_token(tokens, Token::LeftParenthesis)?;

    let mut params = Vec::new();
    let mut found_comma = false;

    loop {
        match tokens.pop_front() {
            None => {
                return Err(vec![TokenStreamError {
                    message: "expected type or )".to_owned(),
                }])
            }
            Some(Token::RightParenthesis) => {
                break;
            }
            Some(Token::Comma) => {
                found_comma = true;
            }
            Some(Token::TypeKeyword(_)) if !found_comma && params.len() > 0 => {
                return Err(vec![TokenStreamError {
                    message: "expected , or )".to_owned(),
                }])
            }
            Some(Token::TypeKeyword(type_)) => {
                match tokens.pop_front() {
                    None => {
                        return Err(vec![TokenStreamError {
                            message: "expected type or )".to_owned(),
                        }])
                    }
                    Some(Token::Identifier(identifier)) => {
                        params.push(FunctionParameter::FunctionParameter {
                            param_type: type_,
                            param_name: identifier,
                        });
                    }
                    Some(_) => {
                        return Err(vec![TokenStreamError {
                            message: "expected type or )".to_owned(),
                        }]);
                    }
                }
                found_comma = false;
            }
            Some(_) => {
                return Err(vec![TokenStreamError {
                    message: "expected type, ',', or )".to_owned(),
                }])
            }
        }
    }

    let mut return_type = None;

    match tokens.pop_front() {
        None => {
            return Err(vec![TokenStreamError {
                message: "Expected -> or {".to_owned(),
            }])
        }
        Some(Token::FunctionSignitureSplitter) => match tokens.pop_front() {
            None => {
                return Err(vec![TokenStreamError {
                    message: "Expected -> or {".to_owned(),
                }]);
            }
            Some(Token::TypeKeyword(type_)) => {
                return_type = Some(type_);
                ensure_token(tokens, Token::LeftCurleyBrace)?;
            }
            Some(_) => {
                return Err(vec![TokenStreamError {
                    message: "Expected -> or {".to_owned(),
                }])
            }
        },
        Some(Token::LeftCurleyBrace) => {}
        Some(_) => {
            return Err(vec![TokenStreamError {
                message: "Expected -> or {".to_owned(),
            }]);
        }
    }

    let statements = get_block_statements(tokens)?;

    Ok(Box::new(move |mut function_declaration_builder| {
        function_declaration_builder = function_declaration_builder
            .name(&function_name)
            .parameters(params);

        if let Some(return_type) = return_type {
            function_declaration_builder = function_declaration_builder.return_type(return_type);
        } else {
            function_declaration_builder = function_declaration_builder.void();
        }

        function_declaration_builder.body(|mut body| {
            for statement in statements {
                body = body.statement(statement);
            }

            body.build()
        })
    }))
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

    #[test]
    fn function_declaration_no_parameters_no_return_value() {
        let tokens = vec![
            Token::FunctionKeyword,
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
            Token::RightParenthesis,
            Token::LeftCurleyBrace,
            Token::ReturnKeyword,
            Token::SemiColon,
            Token::RightCurleyBrace,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        let expected = AstBuilder::default().function_declaration(|function_declaration| {
            function_declaration
                .name("my_function")
                .parameters(Vec::new())
                .void()
                .body(|body| body.statement(|statement| statement.return_void()).build())
        });

        assert!(matches!(result, Ok(ast_builder) if ast_builder == expected));
    }

    #[test]
    fn function_declaration_parameters_no_return_value() {
        let tokens = vec![
            Token::FunctionKeyword,
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
            Token::TypeKeyword(Type::UInt),
            Token::Identifier("param_1".to_owned()),
            Token::Comma,
            Token::TypeKeyword(Type::Boolean),
            Token::Identifier("param_2".to_owned()),
            Token::RightParenthesis,
            Token::LeftCurleyBrace,
            Token::ReturnKeyword,
            Token::SemiColon,
            Token::RightCurleyBrace,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        let expected = AstBuilder::default().function_declaration(|function_declaration| {
            function_declaration
                .name("my_function")
                .parameters(vec![
                    FunctionParameter::FunctionParameter {
                        param_type: Type::UInt,
                        param_name: "param_1".to_owned(),
                    },
                    FunctionParameter::FunctionParameter {
                        param_type: Type::Boolean,
                        param_name: "param_2".to_owned(),
                    },
                ])
                .void()
                .body(|body| body.statement(|statement| statement.return_void()).build())
        });

        assert!(matches!(result, Ok(ast_builder) if ast_builder == expected));
    }

    #[test]
    fn function_declaration_parameters_return_value() {
        let tokens = vec![
            Token::FunctionKeyword,
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
            Token::TypeKeyword(Type::UInt),
            Token::Identifier("param_1".to_owned()),
            Token::Comma,
            Token::TypeKeyword(Type::Boolean),
            Token::Identifier("param_2".to_owned()),
            Token::RightParenthesis,
            Token::FunctionSignitureSplitter,
            Token::TypeKeyword(Type::Boolean),
            Token::LeftCurleyBrace,
            Token::ReturnKeyword,
            Token::SemiColon,
            Token::RightCurleyBrace,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        let expected = AstBuilder::default().function_declaration(|function_declaration| {
            function_declaration
                .name("my_function")
                .parameters(vec![
                    FunctionParameter::FunctionParameter {
                        param_type: Type::UInt,
                        param_name: "param_1".to_owned(),
                    },
                    FunctionParameter::FunctionParameter {
                        param_type: Type::Boolean,
                        param_name: "param_2".to_owned(),
                    },
                ])
                .return_type(Type::Boolean)
                .body(|body| body.statement(|statement| statement.return_void()).build())
        });

        assert!(matches!(result, Ok(ast_builder) if ast_builder == expected));
    }
}
