use std::collections::VecDeque;

use crate::ast::{builders::function_call_builder::FunctionCallBuilder, node::FunctionCall};

use super::{
    expression::create_expression,
    token::{ensure_token, Token, TokenStreamError},
};

pub(super) fn take_function_call(
    identifier: String,
    tokens: &mut VecDeque<Token>,
) -> Result<Box<dyn FnOnce(FunctionCallBuilder) -> FunctionCall>, Vec<TokenStreamError>> {
    ensure_token(tokens, Token::LeftParenthesis)?;

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
                return Ok(Box::new(move |mut function_call| {
                    function_call = function_call.function_id(&identifier);
                    if params.is_empty() {
                        function_call = function_call.no_parameters();
                    } else {
                        while let Some(param) = params.pop_front() {
                            function_call = function_call.parameter(param);
                        }
                    }
                    function_call.build()
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
                params.push_back(create_expression(tokens)?);
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::{ast::builders::ast_builder::AstBuilder, token_stream::token::Token};

    #[test]
    fn function_call_missing_left_parenthesis() {
        let tokens = vec![Token::Identifier("my_function".to_owned())];

        let result = AstBuilder::from_token_stream(tokens);

        assert!(
            matches!(result, Err(e) if e.len() == 1 && e[0].message == "Expected LeftParenthesis")
        )
    }

    #[test]
    fn function_call_missing_right_parenthesis() {
        let tokens = vec![
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        assert!(
            matches!(result, Err(e) if e.len() == 1 && e[0].message == "unexpected end of function call")
        )
    }

    #[test]
    fn function_call_missing_comma() {
        let tokens = vec![
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
            Token::TrueKeyword,
            Token::LeftCurleyBrace,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        assert!(
            matches!(dbg!(result), Err(e) if e.len() == 1 && e[0].message == "Require comma separating parameters".to_owned())
        )
    }
}
