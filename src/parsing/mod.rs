use std::str::FromStr;

use crate::{ast::node::Type, token_stream::token::Token};

fn push_current_buffer(tokens: &mut Vec<Token>, buffer: &mut String) {
    if let Ok(token) = buffer.parse() {
        tokens.push(token);
    }
    *buffer = String::new();
}

pub fn parse_program(code: &str) -> Result<Vec<Token>, Vec<String>> {
    let mut tokens = Vec::new();
    let mut buffer = String::new();

    for char in code.chars() {
        match char {
            '(' => {
                push_current_buffer(&mut tokens, &mut buffer);
                tokens.push(Token::LeftParenthesis)
            }
            ')' => {
                push_current_buffer(&mut tokens, &mut buffer);
                tokens.push(Token::RightParenthesis)
            }
            '{' => {
                push_current_buffer(&mut tokens, &mut buffer);
                tokens.push(Token::LeftCurleyBrace);
            }
            '}' => {
                push_current_buffer(&mut tokens, &mut buffer);
                tokens.push(Token::RightCurleyBrace);
            }
            '+' => {
                push_current_buffer(&mut tokens, &mut buffer);
                tokens.push(Token::PlusOperator);
            }
            '=' => {
                push_current_buffer(&mut tokens, &mut buffer);
                tokens.push(Token::AssignmentOperator);
            }
            '>' => {
                push_current_buffer(&mut tokens, &mut buffer);
                tokens.push(Token::RightArrow);
            }
            '!' => {
                push_current_buffer(&mut tokens, &mut buffer);
                tokens.push(Token::NotOperator);
            }
            _ if char.is_whitespace() => {
                push_current_buffer(&mut tokens, &mut buffer);
            }
            _ if char.is_alphanumeric() => {
                buffer.push(char);
            }
            _ => return Err(vec![format!("Unexpected character {}", char)]),
        }
    }

    push_current_buffer(&mut tokens, &mut buffer);

    Ok(tokens)
}

impl FromStr for Token {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        match trimmed {
            "" => Err(()),
            "uint" => Ok(Token::TypeKeyword(Type::UInt)),
            "boolean" => Ok(Token::TypeKeyword(Type::Boolean)),
            "true" => Ok(Token::TrueKeyword),
            "false" => Ok(Token::FalseKeyword),
            "function" => Ok(Token::FunctionKeyword),
            "infer" => Ok(Token::InferKeyword),
            "if" => Ok(Token::IfKeyword),
            "else" => Ok(Token::ElseKeyword),
            "return" => Ok(Token::ReturnKeyword),
            _ => Ok(Token::Identifier(trimmed.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{ast::node::Type, token_stream::token::Token};

    use super::parse_program;

    #[test]
    fn parse_keywords() {
        let code = "uint boolean true false function infer if else return";
        let result = parse_program(code);

        assert_eq!(
            result,
            Ok(vec![
                Token::TypeKeyword(Type::UInt),
                Token::TypeKeyword(Type::Boolean),
                Token::TrueKeyword,
                Token::FalseKeyword,
                Token::FunctionKeyword,
                Token::InferKeyword,
                Token::IfKeyword,
                Token::ElseKeyword,
                Token::ReturnKeyword
            ])
        );
    }

    #[test]
    fn parse_special_tokens() {
        let code = "(){}+>!=";
        let result = parse_program(code);

        assert_eq!(
            result,
            Ok(vec![
                Token::LeftParenthesis,
                Token::RightParenthesis,
                Token::LeftCurleyBrace,
                Token::RightCurleyBrace,
                Token::PlusOperator,
                Token::RightArrow,
                Token::NotOperator,
                Token::AssignmentOperator
            ])
        );
    }

    #[test]
    fn parse_identifier() {
        let code = "uint myIdentifier0";
        let result = parse_program(code);

        assert_eq!(
            result,
            Ok(vec![
                Token::TypeKeyword(Type::UInt),
                Token::Identifier("myIdentifier0".to_owned())
            ])
        );
    }
}
