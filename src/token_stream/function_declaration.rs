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
                message: format!("expected function name. found {}", token),
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
                            message: "expected parameter name".to_owned(),
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
                            message: "expected parameter name".to_owned(),
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
                    message: "Expected return type".to_owned(),
                }]);
            }
            Some(Token::TypeKeyword(type_)) => {
                return_type = Some(type_);
                ensure_token(tokens, Token::LeftCurleyBrace)?;
            }
            Some(_) => {
                return Err(vec![TokenStreamError {
                    message: "Expected return type".to_owned(),
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

    /// function my_function()
    /// {
    ///     print(1);
    ///     return;
    /// }
    #[test]
    fn function_declaration_no_parameters_no_return_value() {
        let tokens = vec![
            Token::FunctionKeyword,
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
            Token::RightParenthesis,
            Token::LeftCurleyBrace,
            Token::Identifier("print".to_owned()),
            Token::LeftParenthesis,
            Token::UIntValue(1),
            Token::RightParenthesis,
            Token::SemiColon,
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
                .body(|body| {
                    body.statement(|statement| {
                        statement.function_call(|function_call| {
                            function_call
                                .function_id("print")
                                .parameter(|_| 1.into())
                                .build()
                        })
                    })
                    .statement(|statement| statement.return_void())
                    .build()
                })
        });

        assert!(matches!(result, Ok(ast_builder) if ast_builder == expected));
    }

    /// function my_function() {}
    #[test]
    fn function_declaration_empty_body() {
        let tokens = vec![
            Token::FunctionKeyword,
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
            Token::RightParenthesis,
            Token::LeftCurleyBrace,
            Token::RightCurleyBrace,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        let expected = AstBuilder::default().function_declaration(|function_declaration| {
            function_declaration
                .name("my_function")
                .parameters(Vec::new())
                .void()
                .body(|body| body.build())
        });

        assert!(matches!(result, Ok(ast_builder) if ast_builder == expected));
    }

    /// function my_function(uint param_1, boolean param_2)
    /// {
    ///     return;
    /// }
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

    /// function my_function(uint param_1, boolean param_2) -> boolean
    /// {
    ///     return true;
    /// }
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
            Token::TrueKeyword,
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
                .body(|body| {
                    body.statement(|statement| statement.return_value(|_| true.into()))
                        .build()
                })
        });

        assert!(matches!(result, Ok(ast_builder) if ast_builder == expected));
    }

    /// function
    #[test]
    fn function_declaration_no_tokens() {
        let tokens = vec![Token::FunctionKeyword];

        let result = AstBuilder::from_token_stream(tokens);

        assert!(
            matches!(result, Err(e) if e.len() == 1 && e[0].message == "expected function name")
        );
    }

    /// function (
    #[test]
    fn function_declaration_invalid_function_name() {
        let tokens = vec![Token::FunctionKeyword, Token::LeftParenthesis];

        let result = AstBuilder::from_token_stream(tokens);

        assert!(
            matches!(result, Err(e) if e.len() == 1 && e[0].message == "expected function name. found LeftParenthesis")
        )
    }

    /// function my_function
    #[test]
    fn function_declaration_missing_left_parenthesis() {
        let tokens = vec![
            Token::FunctionKeyword,
            Token::Identifier("my_function".to_owned()),
            Token::RightParenthesis,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        assert!(
            matches!(result, Err(e) if e.len() == 1 && e[0].message == "Expected LeftParenthesis, found RightParenthesis")
        );
    }

    /// function my_function
    #[test]
    fn function_declaration_missing_left_parenthesis_no_tokens() {
        let tokens = vec![
            Token::FunctionKeyword,
            Token::Identifier("my_function".to_owned()),
        ];

        let result = AstBuilder::from_token_stream(tokens);

        assert!(
            matches!(result, Err(e) if e.len() == 1 && e[0].message == "Expected LeftParenthesis")
        );
    }

    /// function my_function(my_param
    #[test]
    fn function_declaration_missing_type() {
        let tokens = vec![
            Token::FunctionKeyword,
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
            Token::Identifier("my_param".to_owned()),
        ];

        let result = AstBuilder::from_token_stream(tokens);

        assert!(
            matches!(result, Err(e) if e.len() == 1 && e[0].message == "expected type, ',', or )")
        );
    }

    /// function my_function(boolean)
    #[test]
    fn function_declaration_missing_param_name() {
        let tokens = vec![
            Token::FunctionKeyword,
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
            Token::TypeKeyword(Type::Boolean),
            Token::RightParenthesis,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        assert!(
            matches!(result, Err(e) if e.len() == 1 && e[0].message == "expected parameter name")
        );
    }

    /// function my_function(boolean
    #[test]
    fn function_declaration_missing_param_name_no_tokens() {
        let tokens = vec![
            Token::FunctionKeyword,
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
            Token::TypeKeyword(Type::Boolean),
        ];

        let result = AstBuilder::from_token_stream(tokens);

        assert!(
            matches!(result, Err(e) if e.len() == 1 && e[0].message == "expected parameter name")
        );
    }

    /// function my_function(boolean my_param boolean other_param)
    #[test]
    fn function_declaration_missing_comma() {
        let tokens = vec![
            Token::FunctionKeyword,
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
            Token::TypeKeyword(Type::Boolean),
            Token::Identifier("my_param".to_owned()),
            Token::TypeKeyword(Type::Boolean),
            Token::Identifier("other_param".to_owned()),
            Token::RightParenthesis,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        assert!(matches!(result, Err(e) if e.len() >= 1 && e[0].message == "expected , or )"));
    }

    /// function my_function( {
    #[test]
    fn function_declaration_missing_right_parenthesis() {
        let tokens = vec![
            Token::FunctionKeyword,
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
            Token::LeftCurleyBrace,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        assert!(
            matches!(result, Err(e) if e.len() == 1 && e[0].message == "expected type, ',', or )")
        );
    }

    /// function my_function(
    #[test]
    fn function_declaration_missing_right_parenthesis_no_tokens() {
        let tokens = vec![
            Token::FunctionKeyword,
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        assert!(matches!(result, Err(e) if e.len() == 1 && e[0].message == "expected type or )"));
    }

    /// function my_function() infer
    #[test]
    fn function_declaration_missing_left_curley_brace() {
        let tokens = vec![
            Token::FunctionKeyword,
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
            Token::RightParenthesis,
            Token::InferKeyword,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        assert!(matches!(result, Err(e) if e.len() == 1 && e[0].message == "Expected -> or {"));
    }

    /// function my_function()
    #[test]
    fn function_declaration_missing_left_curley_brace_no_tokens() {
        let tokens = vec![
            Token::FunctionKeyword,
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
            Token::RightParenthesis,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        assert!(matches!(result, Err(e) if e.len() == 1 && e[0].message == "Expected -> or {"));
    }

    /// function my_function () { return;
    /// function
    #[test]
    fn function_declaration_missing_right_curley_brace() {
        let tokens = vec![
            Token::FunctionKeyword,
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
            Token::RightParenthesis,
            Token::LeftCurleyBrace,
            Token::ReturnKeyword,
            Token::SemiColon,
            Token::FunctionKeyword,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        assert!(matches!(result, Err(e) if e.len() == 1 && e[0].message == "Expected }"));
    }

    /// function my_function() -> {
    #[test]
    fn function_declaration_missing_return_type() {
        let tokens = vec![
            Token::FunctionKeyword,
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
            Token::RightParenthesis,
            Token::FunctionSignitureSplitter,
            Token::LeftCurleyBrace,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        assert!(matches!(result, Err(e) if e.len() == 1 && e[0].message == "Expected return type"));
    }

    /// function my_function() ->
    #[test]
    fn function_declaration_missing_return_type_no_tokens() {
        let tokens = vec![
            Token::FunctionKeyword,
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
            Token::RightParenthesis,
            Token::FunctionSignitureSplitter,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        assert!(matches!(result, Err(e) if e.len() == 1 && e[0].message == "Expected return type"));
    }
}
