use std::str::FromStr;

use crate::{
    ast::node::Type,
    token_stream::token::{Token, TokenSource},
};

fn push_current_buffer(
    tokens: &mut Vec<(Token, TokenSource)>,
    buffer: &mut (Option<u32>, u32, String),
) -> Result<(), Vec<String>> {
    let len = buffer.2.len() as u32;
    match buffer.2.parse() {
        Ok(token) => {
            let character_start = if let Some(character_start) = buffer.0 {
                character_start
            } else {
                0
            };

            tokens.push((
                token,
                TokenSource::new(
                    "my_file",
                    buffer.1,
                    character_start..(character_start + len - 1),
                ),
            ))
        }
        Err(Some(error)) => return Err(vec![error]),
        Err(None) => {}
    }
    *buffer = (None, buffer.1, String::new());
    Ok(())
}

pub fn parse_program(code: &str) -> Result<Vec<(Token, TokenSource)>, Vec<String>> {
    let mut tokens = Vec::new();
    let mut buffer = (None, 0, String::new());

    fn new_buffer(
        buffer: (Option<u32>, u32, String),
        char: char,
        column: u32,
        line: u32,
    ) -> (Option<u32>, u32, String) {
        let (current_column, current_line, mut buffer_str) = buffer;
        let (new_column, new_line) = if current_column.is_some() {
            (current_column, current_line)
        } else {
            (Some(column), line)
        };

        buffer_str.push(char);

        (new_column, new_line, buffer_str)
    }

    for (line_index, line) in code.lines().enumerate() {
        let line_index = (line_index + 1) as u32;
        for (column, char) in line.char_indices() {
            let column = (column + 1) as u32;
            match char {
                '=' => {
                    push_current_buffer(&mut tokens, &mut buffer)?;
                    buffer = new_buffer(buffer, '=', column, line_index);
                    push_current_buffer(&mut tokens, &mut buffer)?;
                }
                '>' => {
                    if buffer.2 == "-" {
                        buffer = new_buffer(buffer, '>', column, line_index);
                        push_current_buffer(&mut tokens, &mut buffer)?;
                    } else {
                        push_current_buffer(&mut tokens, &mut buffer)?;
                        buffer = new_buffer(buffer, '>', column, line_index);
                    }
                }
                _ if char.is_whitespace() => {
                    push_current_buffer(&mut tokens, &mut buffer)?;
                }
                _ if char.is_ascii_punctuation() && char != '_' => {
                    push_current_buffer(&mut tokens, &mut buffer)?;
                    buffer = new_buffer(buffer, char, column, line_index);
                }
                _ => {
                    if buffer.2.len() == 1 {
                        let char = buffer.2.chars().next().unwrap();
                        if char.is_ascii_punctuation() && char != '_' {
                            push_current_buffer(&mut tokens, &mut buffer)?;
                        }
                    }
                    buffer = new_buffer(buffer, char, column, line_index);
                }
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
    use crate::{
        ast::node::Type,
        token_stream::token::{Token, TokenSource},
    };

    use super::parse_program;

    const FILENAME: &str = "my_file";

    fn get_range(prev_character: &mut Option<u32>, word: &str, space: bool) -> TokenSource {
        let start_char = if let Some(prev_character) = prev_character {
            *prev_character + if space { 2 } else { 1 }
        } else {
            1
        };

        let end_char = start_char + (word.len() as u32) - 1;
        *prev_character = Some(end_char);

        TokenSource::new(FILENAME, 1, start_char..(end_char))
    }

    fn get_range_with_line(
        prev_character: &mut Option<u32>,
        prev_line: &mut Option<u32>,
        word: &str,
        increase_lines: u32,
        space: bool,
    ) -> TokenSource {
        let start_line = if let Some(prev_line) = prev_line {
            *prev_line
        } else {
            1
        };

        let start_char = if let Some(prev_character) = prev_character {
            if increase_lines > 0 {
                1
            } else {
                *prev_character + if space { 2 } else { 1 }
            }
        } else {
            1
        };

        let end_line = start_line + increase_lines;
        *prev_line = Some(end_line);

        let end_char = start_char + (word.len() as u32) - 1;
        *prev_character = Some(end_char);

        TokenSource::new(FILENAME, end_line, start_char..end_char)
    }

    #[test]
    fn parse_keywords() {
        let code = "uint boolean true false function infer if else return";
        let result = parse_program(code);

        let mut prev_character = None;

        assert_eq!(
            result,
            Ok(vec![
                (
                    Token::TypeKeyword(Type::UInt),
                    get_range(&mut prev_character, "uint", true)
                ),
                (
                    Token::TypeKeyword(Type::Boolean),
                    get_range(&mut prev_character, "boolean", true)
                ),
                (
                    Token::TrueKeyword,
                    get_range(&mut prev_character, "true", true)
                ),
                (
                    Token::FalseKeyword,
                    get_range(&mut prev_character, "false", true)
                ),
                (
                    Token::FunctionKeyword,
                    get_range(&mut prev_character, "function", true)
                ),
                (
                    Token::InferKeyword,
                    get_range(&mut prev_character, "infer", true)
                ),
                (Token::IfKeyword, get_range(&mut prev_character, "if", true)),
                (
                    Token::ElseKeyword,
                    get_range(&mut prev_character, "else", true)
                ),
                (
                    Token::ReturnKeyword,
                    get_range(&mut prev_character, "return", true)
                )
            ])
        );
    }

    #[test]
    fn parse_special_tokens() {
        let code = "(){}+>!=;,";
        let result = parse_program(code);

        let mut prev_character = None;

        assert_eq!(
            result,
            Ok(vec![
                (
                    Token::LeftParenthesis,
                    get_range(&mut prev_character, "(", false)
                ),
                (
                    Token::RightParenthesis,
                    get_range(&mut prev_character, ")", false)
                ),
                (
                    Token::LeftCurleyBrace,
                    get_range(&mut prev_character, "{", false)
                ),
                (
                    Token::RightCurleyBrace,
                    get_range(&mut prev_character, "}", false)
                ),
                (
                    Token::PlusOperator,
                    get_range(&mut prev_character, "+", false)
                ),
                (
                    Token::RightAngle,
                    get_range(&mut prev_character, ">", false)
                ),
                (
                    Token::NotOperator,
                    get_range(&mut prev_character, "!", false)
                ),
                (
                    Token::AssignmentOperator,
                    get_range(&mut prev_character, "=", false)
                ),
                (Token::SemiColon, get_range(&mut prev_character, ";", false)),
                (Token::Comma, get_range(&mut prev_character, ",", false)),
            ])
        );
    }

    #[test]
    fn parse_identifier() {
        let code = "uint myIdentifier0";
        let result = parse_program(code);

        let mut prev_character = None;

        assert_eq!(
            result,
            Ok(vec![
                (
                    Token::TypeKeyword(Type::UInt),
                    get_range(&mut prev_character, "uint", false)
                ),
                (
                    Token::Identifier("myIdentifier0".to_owned()),
                    get_range(&mut prev_character, "myIdentifier0", true)
                )
            ])
        );
    }

    #[test]
    fn parse_function_splitter() {
        let code = "hello ->";
        let result = parse_program(code);

        let mut prev_character = None;

        assert_eq!(
            result,
            Ok(vec![
                (
                    Token::Identifier("hello".to_owned()),
                    get_range(&mut prev_character, "hello", false)
                ),
                (
                    Token::FunctionSignitureSplitter,
                    get_range(&mut prev_character, "->", true)
                )
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

        let mut prev_character = None;
        let mut prev_line = None;

        let result = parse_program(code);

        assert_eq!(
            result,
            Ok(vec![
                (
                    Token::FunctionKeyword,
                    get_range_with_line(&mut prev_character, &mut prev_line, "function", 0, false)
                ),
                (
                    Token::Identifier("my_function".to_owned()),
                    get_range_with_line(
                        &mut prev_character,
                        &mut prev_line,
                        "my_function",
                        0,
                        true
                    )
                ),
                (
                    Token::LeftParenthesis,
                    get_range_with_line(&mut prev_character, &mut prev_line, "(", 0, false)
                ),
                (
                    Token::TypeKeyword(Type::UInt),
                    get_range_with_line(&mut prev_character, &mut prev_line, "uint", 0, false)
                ),
                (
                    Token::Identifier("param_1".to_owned()),
                    get_range_with_line(&mut prev_character, &mut prev_line, "param_1", 0, true)
                ),
                (
                    Token::Comma,
                    get_range_with_line(&mut prev_character, &mut prev_line, ",", 0, false)
                ),
                (
                    Token::TypeKeyword(Type::Boolean),
                    get_range_with_line(&mut prev_character, &mut prev_line, "boolean", 0, true)
                ),
                (
                    Token::Identifier("param_2".to_owned()),
                    get_range_with_line(&mut prev_character, &mut prev_line, "param_2", 0, true)
                ),
                (
                    Token::RightParenthesis,
                    get_range_with_line(&mut prev_character, &mut prev_line, ")", 0, false)
                ),
                (
                    Token::FunctionSignitureSplitter,
                    get_range_with_line(&mut prev_character, &mut prev_line, "->", 0, true)
                ),
                (
                    Token::TypeKeyword(Type::UInt),
                    get_range_with_line(&mut prev_character, &mut prev_line, "uint", 0, true)
                ),
                (
                    Token::LeftCurleyBrace,
                    get_range_with_line(&mut prev_character, &mut prev_line, "{", 1, false)
                ),
                (
                    Token::InferKeyword,
                    get_range_with_line(&mut prev_character, &mut prev_line, "infer", 1, false)
                ),
                (
                    Token::Identifier("my_var".to_owned()),
                    get_range_with_line(&mut prev_character, &mut prev_line, "my_var", 0, true)
                ),
                (
                    Token::AssignmentOperator,
                    get_range_with_line(&mut prev_character, &mut prev_line, "=", 0, true)
                ),
                (
                    Token::Identifier("other_function".to_owned()),
                    get_range_with_line(
                        &mut prev_character,
                        &mut prev_line,
                        "other_function",
                        0,
                        true
                    )
                ),
                (
                    Token::LeftParenthesis,
                    get_range_with_line(&mut prev_character, &mut prev_line, "(", 0, false)
                ),
                (
                    Token::UIntValue(15),
                    get_range_with_line(&mut prev_character, &mut prev_line, "15", 0, false)
                ),
                (
                    Token::Comma,
                    get_range_with_line(&mut prev_character, &mut prev_line, ",", 0, false)
                ),
                (
                    Token::TrueKeyword,
                    get_range_with_line(&mut prev_character, &mut prev_line, "true", 0, true)
                ),
                (
                    Token::Comma,
                    get_range_with_line(&mut prev_character, &mut prev_line, ",", 0, false)
                ),
                (
                    Token::FalseKeyword,
                    get_range_with_line(&mut prev_character, &mut prev_line, "false", 0, true)
                ),
                (
                    Token::RightParenthesis,
                    get_range_with_line(&mut prev_character, &mut prev_line, ")", 0, false)
                ),
                (
                    Token::SemiColon,
                    get_range_with_line(&mut prev_character, &mut prev_line, ";", 0, false)
                ),
                (
                    Token::ReturnKeyword,
                    get_range_with_line(&mut prev_character, &mut prev_line, "return", 1, false)
                ),
                (
                    Token::Identifier("my_var".to_owned()),
                    get_range_with_line(&mut prev_character, &mut prev_line, "my_var", 0, true)
                ),
                (
                    Token::SemiColon,
                    get_range_with_line(&mut prev_character, &mut prev_line, ";", 0, false)
                ),
                (
                    Token::RightCurleyBrace,
                    get_range_with_line(&mut prev_character, &mut prev_line, "}", 1, false)
                )
            ])
        );
    }

    #[test]
    fn parse_invalid_symbol() {
        let code = "#";

        let result = parse_program(code);
        assert!(matches!(result, Err(_)));
    }
}
