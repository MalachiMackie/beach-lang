use std::collections::VecDeque;

use crate::ast::{
    builders::function_declaration_builder::FunctionDeclarationBuilder, node::FunctionDeclaration,
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
    ensure_token(tokens, Token::RightParenthesis)?;
    ensure_token(tokens, Token::LeftCurleyBrace)?;

    let statements = get_block_statements(tokens)?;

    Ok(Box::new(move |function_declaration_builder| {
        function_declaration_builder
            .name(&function_name)
            .no_parameters()
            .void()
            .body(|mut body| {
                for statement in statements {
                    body = body.statement(statement);
                }

                body.build()
            })
    }))
}

#[cfg(test)]
mod tests {
    use crate::{ast::builders::ast_builder::AstBuilder, token_stream::token::Token};

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
                .no_parameters()
                .void()
                .body(|body| body.statement(|statement| statement.return_void()).build())
        });

        assert!(matches!(result, Ok(ast_builder) if ast_builder == expected));
    }
}
