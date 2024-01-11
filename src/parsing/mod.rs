use std::str::FromStr;

use crate::{ast::node::Type, token_stream::token::Token};

fn push_current_buffer(tokens: &mut Vec<Token>, buffer: &mut String) -> Result<(), Vec<String>> {
    match buffer.parse() {
        Ok(token) => tokens.push(token),
        Err(Some(error)) => return Err(vec![error]),
        Err(None) => {}
    }
    *buffer = String::new();
    Ok(())
}

pub fn parse_program(code: &str) -> Result<Vec<Token>, Vec<String>> {
    let mut tokens = Vec::new();
    let mut buffer = String::new();

    for char in code.chars() {
        match char {
            '=' => {
                push_current_buffer(&mut tokens, &mut buffer)?;
                tokens.push(Token::AssignmentOperator);
            }
            '>' => {
                if buffer == "-" {
                    buffer.push('>');
                    push_current_buffer(&mut tokens, &mut buffer)?;
                } else {
                    push_current_buffer(&mut tokens, &mut buffer)?;
                    buffer.push('>');
                }
            }
            _ if char.is_whitespace() => {
                push_current_buffer(&mut tokens, &mut buffer)?;
            }
            _ if char.is_ascii_punctuation() && char != '_' => {
                push_current_buffer(&mut tokens, &mut buffer)?;
                buffer.push(char);
            }
            _ => {
                if buffer.len() == 1 {
                    let char = buffer.chars().next().unwrap();
                    if char.is_ascii_punctuation() && char != '_' {
                        push_current_buffer(&mut tokens, &mut buffer)?;
                    }
                }
                buffer.push(char);
            }
        }
    }

    push_current_buffer(&mut tokens, &mut buffer)?;

    Ok(tokens)
}

impl FromStr for Token {
    type Err = Option<String>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        match trimmed {
            "" => Err(None),
            "uint" => Ok(Token::TypeKeyword(Type::UInt)),
            "boolean" => Ok(Token::TypeKeyword(Type::Boolean)),
            "true" => Ok(Token::TrueKeyword),
            "false" => Ok(Token::FalseKeyword),
            "function" => Ok(Token::FunctionKeyword),
            "infer" => Ok(Token::InferKeyword),
            "if" => Ok(Token::IfKeyword),
            "else" => Ok(Token::ElseKeyword),
            "return" => Ok(Token::ReturnKeyword),
            "=" => Ok(Token::AssignmentOperator),
            "(" => Ok(Token::LeftParenthesis),
            ")" => Ok(Token::RightParenthesis),
            "{" => Ok(Token::LeftCurleyBrace),
            "}" => Ok(Token::RightCurleyBrace),
            ">" => Ok(Token::RightAngle),
            "!" => Ok(Token::NotOperator),
            "+" => Ok(Token::PlusOperator),
            ";" => Ok(Token::SemiColon),
            "," => Ok(Token::Comma),
            "->" => Ok(Token::FunctionSignitureSplitter),
            _ if s.len() == 1 && s.chars().next().unwrap().is_ascii_punctuation() && s != "_" => {
                Err(Some(format!("Unexpected character `{s}`")))
            }
            _ => {
                if let Ok(u32_value) = s.parse::<u32>() {
                    Ok(Token::UIntValue(u32_value))
                } else {
                    Ok(Token::Identifier(trimmed.to_owned()))
                }
            }
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
        let code = "(){}+>!=;,";
        let result = parse_program(code);

        assert_eq!(
            result,
            Ok(vec![
                Token::LeftParenthesis,
                Token::RightParenthesis,
                Token::LeftCurleyBrace,
                Token::RightCurleyBrace,
                Token::PlusOperator,
                Token::RightAngle,
                Token::NotOperator,
                Token::AssignmentOperator,
                Token::SemiColon,
                Token::Comma,
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

    #[test]
    fn parse_function_splitter() {
        let code = "hello ->";
        let result = parse_program(code);

        assert_eq!(
            result,
            Ok(vec![
                Token::Identifier("hello".to_owned()),
                Token::FunctionSignitureSplitter
            ])
        );
    }

    #[test]
    fn parse_function_declaration() {
        let code = "function my_function(uint param_1, boolean param_2) -> uint
        {
            infer my_var = other_function(15, true, false);
            return my_var;
        }";

        let result = parse_program(code);

        assert_eq!(
            result,
            Ok(vec![
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
                Token::TypeKeyword(Type::UInt),
                Token::LeftCurleyBrace,
                Token::InferKeyword,
                Token::Identifier("my_var".to_owned()),
                Token::AssignmentOperator,
                Token::Identifier("other_function".to_owned()),
                Token::LeftParenthesis,
                Token::UIntValue(15),
                Token::Comma,
                Token::TrueKeyword,
                Token::Comma,
                Token::FalseKeyword,
                Token::RightParenthesis,
                Token::SemiColon,
                Token::ReturnKeyword,
                Token::Identifier("my_var".to_owned()),
                Token::SemiColon,
                Token::RightCurleyBrace
            ])
        );
    }
}
