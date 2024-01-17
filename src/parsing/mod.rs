use std::ops::Range;

use crate::{
    ast::node::Type,
    token_stream::token::{Token, TokenSource},
};

#[derive(PartialEq, Debug)]
pub struct ParseError {
    pub error: String,
    pub file: String,
    pub line: u32,
    pub character_range: Range<u32>,
}

#[derive(Default)]
struct Buffer {
    character_start: Option<u32>,
    line: u32,
    value: String,
}

fn push_current_buffer(
    tokens: &mut Vec<(Token, TokenSource)>,
    buffer: &mut Buffer,
) -> Result<(), Vec<ParseError>> {
    let len = buffer.value.len() as u32;
    let character_start = buffer.character_start.unwrap_or(0);
    if len > 0 {
        match Token::from_str(
            &buffer.value,
            "my_file",
            buffer.line,
            character_start..(character_start + len - 1),
        ) {
            Ok(Some(token)) => {
                let character_start = if let Some(character_start) = buffer.character_start {
                    character_start
                } else {
                    0
                };

                tokens.push((
                    token,
                    TokenSource::new(
                        "my_file",
                        buffer.line,
                        character_start..(character_start + len - 1),
                    ),
                ))
            }
            Ok(None) => {}
            Err(error) => return Err(vec![error]),
        }
    }
    *buffer = Buffer {
        character_start: None,
        line: buffer.line,
        value: String::new(),
    };
    Ok(())
}

impl Buffer {
    fn update(&mut self, char: char, column: u32, line: u32) {
        let (new_column, new_line) = if let Some(character_start) = self.character_start {
            (character_start, self.line)
        } else {
            (column, line)
        };

        self.character_start = Some(new_column);
        self.line = new_line;

        self.value.push(char);
    }
}

pub fn parse_program(code: &str) -> Result<Vec<(Token, TokenSource)>, Vec<ParseError>> {
    let mut tokens = Vec::new();
    let mut buffer = Buffer::default();

    for (line_index, line) in code.lines().enumerate() {
        let line_index = (line_index + 1) as u32;
        for (column, char) in line.char_indices() {
            let column = (column + 1) as u32;
            match char {
                '=' => {
                    push_current_buffer(&mut tokens, &mut buffer)?;
                    buffer.update('=', column, line_index);
                    push_current_buffer(&mut tokens, &mut buffer)?;
                }
                '>' => {
                    if buffer.value == "-" {
                        buffer.update('>', column, line_index);
                        push_current_buffer(&mut tokens, &mut buffer)?;
                    } else {
                        push_current_buffer(&mut tokens, &mut buffer)?;
                        buffer.update('>', column, line_index);
                    }
                }
                _ if char.is_whitespace() => {
                    push_current_buffer(&mut tokens, &mut buffer)?;
                }
                _ if char.is_ascii_punctuation() && char != '_' => {
                    push_current_buffer(&mut tokens, &mut buffer)?;
                    buffer.update(char, column, line_index);
                }
                _ => {
                    if buffer.value.len() == 1 {
                        let char = buffer.value.chars().next().unwrap();
                        if char.is_ascii_punctuation() && char != '_' {
                            push_current_buffer(&mut tokens, &mut buffer)?;
                        }
                    }
                    buffer.update(char, column, line_index);
                }
            }
        }
    }

    push_current_buffer(&mut tokens, &mut buffer)?;

    Ok(tokens)
}

impl Token {
    fn from_str(
        s: &str,
        file: &str,
        line: u32,
        character_range: Range<u32>,
    ) -> Result<Option<Self>, ParseError> {
        let trimmed = s.trim();
        match trimmed {
            "" => Ok(None),
            "uint" => Ok(Some(Token::TypeKeyword(Type::UInt))),
            "boolean" => Ok(Some(Token::TypeKeyword(Type::Boolean))),
            "true" => Ok(Some(Token::TrueKeyword)),
            "false" => Ok(Some(Token::FalseKeyword)),
            "function" => Ok(Some(Token::FunctionKeyword)),
            "infer" => Ok(Some(Token::InferKeyword)),
            "if" => Ok(Some(Token::IfKeyword)),
            "else" => Ok(Some(Token::ElseKeyword)),
            "return" => Ok(Some(Token::ReturnKeyword)),
            "=" => Ok(Some(Token::AssignmentOperator)),
            "(" => Ok(Some(Token::LeftParenthesis)),
            ")" => Ok(Some(Token::RightParenthesis)),
            "{" => Ok(Some(Token::LeftCurleyBrace)),
            "}" => Ok(Some(Token::RightCurleyBrace)),
            ">" => Ok(Some(Token::RightAngle)),
            "!" => Ok(Some(Token::NotOperator)),
            "+" => Ok(Some(Token::PlusOperator)),
            ";" => Ok(Some(Token::SemiColon)),
            "," => Ok(Some(Token::Comma)),
            "->" => Ok(Some(Token::FunctionSignitureSplitter)),
            _ if s.len() == 1 && s.chars().next().unwrap().is_ascii_punctuation() && s != "_" => {
                Err(ParseError {
                    error: format!("Unexpected character `{s}`"),
                    file: file.to_owned(),
                    character_range,
                    line,
                })
            }
            _ => {
                if let Ok(u32_value) = s.parse::<u32>() {
                    Ok(Some(Token::UIntValue(u32_value)))
                } else {
                    Ok(Some(Token::Identifier(trimmed.to_owned())))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::node::Type,
        parsing::ParseError,
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
        let code = "   #";

        let result = parse_program(code);
        assert!(
            matches!(result, Err(err) if err == vec![ParseError{file: FILENAME.to_owned(), error: "Unexpected character `#`".to_owned(), line: 1, character_range: 4..4}])
        );
    }
}
