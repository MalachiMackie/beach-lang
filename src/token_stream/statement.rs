use std::collections::VecDeque;

use crate::ast::{
    builders::{expression_builder::ExpressionBuilder, statement_builder::StatementBuilder},
    node::{Expression, Node, VariableDeclarationType},
};

use super::{
    expression::create_expression,
    function_call::take_function_call,
    if_statement::try_create_if_statement,
    token::{ensure_token, take_from_front_while, Token, TokenStreamError},
    variable_declaration::try_create_variable_declaration,
};

pub(super) fn try_create_statement(
    first_token: Token,
    tokens: &mut VecDeque<Token>,
) -> Result<Option<Box<dyn FnOnce(StatementBuilder) -> Node>>, Vec<TokenStreamError>> {
    let statement_type = match first_token {
        Token::Identifier(identifier) => StatementType::FunctionCall(identifier),
        Token::TypeKeyword(type_) => {
            StatementType::VariableDeclaration(VariableDeclarationType::Type(type_))
        }
        Token::InferKeyword => StatementType::VariableDeclaration(VariableDeclarationType::Infer),
        Token::IfKeyword => StatementType::If,
        Token::ReturnKeyword => StatementType::Return,
        _ => {
            tokens.push_front(first_token);
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

            let var_decl_builder = try_create_variable_declaration(var_decl_type, tokens)?;
            Ok(Box::new(|statement_builder: StatementBuilder| {
                statement_builder.var_declaration(var_decl_builder)
            }))
        }
        StatementType::FunctionCall(identifier) => {
            Ok(Box::new(take_function_call_statement(identifier, tokens)?))
        }
        StatementType::If => {
            let if_statement_builder = try_create_if_statement(tokens)?;
            Ok(Box::new(|statement_builder: StatementBuilder| {
                statement_builder.if_statement(if_statement_builder)
            }))
        }
        StatementType::Return => Ok(Box::new(take_return_statement(tokens)?)),
    }
}

fn take_function_call_statement(
    identifier: String,
    tokens: &mut VecDeque<Token>,
) -> Result<Box<dyn FnOnce(StatementBuilder) -> Node>, Vec<TokenStreamError>> {
    let function_call = take_function_call(identifier, tokens)?;

    ensure_token(tokens, Token::SemiColon)?;

    Ok(Box::new(|statement_builder| {
        statement_builder.function_call(function_call)
    }))
}

fn take_return_statement(
    tokens: &mut VecDeque<Token>,
) -> Result<Box<dyn FnOnce(StatementBuilder) -> Node>, Vec<TokenStreamError>> {
    match tokens.pop_front() {
        None => Err(vec![TokenStreamError {
            message: "expected ;".to_owned(),
        }]),
        Some(Token::SemiColon) => Ok(build_return_statement(None)),
        Some(token) => {
            tokens.push_front(token);
            let expression = create_expression(tokens)?;

            ensure_token(tokens, Token::SemiColon)?;

            Ok(build_return_statement(Some(expression)))
        }
    }
}

fn build_return_statement(
    expression: Option<Box<dyn FnOnce(ExpressionBuilder) -> Expression>>,
) -> Box<dyn FnOnce(StatementBuilder) -> Node> {
    if let Some(expression) = expression {
        Box::new(|statement_builder: StatementBuilder| statement_builder.return_value(expression))
    } else {
        Box::new(|statement_builder: StatementBuilder| statement_builder.return_void())
    }
}

#[cfg(test)]
mod tests {
    use crate::{ast::builders::ast_builder::AstBuilder, token_stream::token::Token};

    /// return;
    #[test]
    fn return_statement() {
        let tokens = vec![Token::ReturnKeyword, Token::SemiColon];

        let result = AstBuilder::from_token_stream(tokens);

        let expected = AstBuilder::default().statement(|statement| statement.return_void());

        assert!(matches!(result, Ok(ast_builder) if ast_builder == expected));
    }

    /// return 1 + 2;
    #[test]
    fn return_statement_value() {
        let tokens = vec![
            Token::ReturnKeyword,
            Token::UIntValue(1),
            Token::PlusOperator,
            Token::UIntValue(2),
            Token::SemiColon,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        let expected = AstBuilder::default().statement(|statement| {
            statement.return_value(|value| {
                value.operation(|operation| operation.plus(|_| 1.into(), |_| 2.into()))
            })
        });

        assert!(matches!(result, Ok(ast_builder) if ast_builder == expected));
    }

    /// print(my_function(true));
    #[test]
    fn function_call_statement() {
        let tokens = vec![
            Token::Identifier("print".to_owned()),
            Token::LeftParenthesis,
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
            Token::TrueKeyword,
            Token::RightParenthesis,
            Token::RightParenthesis,
            Token::SemiColon,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        let expected = AstBuilder::default().statement(|statement| {
            statement.function_call(|function_call| {
                function_call
                    .function_id("print")
                    .parameter(|param| {
                        param.function_call(|function_call| {
                            function_call
                                .function_id("my_function")
                                .parameter(|_| true.into())
                                .build()
                        })
                    })
                    .build()
            })
        });

        assert!(matches!(result, Ok(ast_builder) if ast_builder == expected));
    }

    #[test]
    fn statement_missing_semicolon() {
        let tokens = vec![Token::InferKeyword];

        let result = AstBuilder::from_token_stream(tokens);

        assert!(matches!(result, Err(e) if e.len() == 1 && e[0].message == "expected ;"))
    }

    #[test]
    fn statement_function_call_expected_semicolon() {
        let tokens = vec![
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
            Token::RightParenthesis,
            Token::TrueKeyword,
            Token::SemiColon,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        assert!(
            matches!(result, Err(e) if e.len() >= 1 && e[0].message == "Expected SemiColon, found TrueKeyword")
        )
    }

    #[test]
    fn statement_return_expected_semicolon() {
        let tokens = vec![
            Token::ReturnKeyword,
            Token::TrueKeyword,
            Token::LeftParenthesis,
            Token::SemiColon,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        assert!(
            matches!(dbg!(result), Err(e) if e.len() >= 1 && e[0].message == "Expected SemiColon, found LeftParenthesis")
        );
    }

    #[test]
    fn statement_return_missing_semicolon() {
        let tokens = vec![Token::ReturnKeyword];


        let result = AstBuilder::from_token_stream(tokens);

        assert!(
            matches!(dbg!(result), Err(e) if e.len() >= 1 && e[0].message == "expected ;")
        );
    }
}
