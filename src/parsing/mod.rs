use crate::{ast::node::Type, token_stream::token::Token};

pub fn parse_program(code: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut buffer = String::new();

    for char in code.chars() {
        if is_word_delimeter(char) {
            if !buffer.is_empty() && buffer.chars().any(|ch| !ch.is_whitespace()) {
                tokens.push(string_to_token(&buffer));
            }
            buffer = String::new();
        }
        buffer.push(char)
    }

    if !buffer.is_empty() && buffer.chars().any(|ch| !ch.is_whitespace()) {
        tokens.push(string_to_token(&buffer));
    }

    tokens
}

fn is_word_delimeter(value: char) -> bool {
    value.is_whitespace() || value.is_ascii_punctuation()
}

fn string_to_token(value: &str) -> Token {
    match value.trim() {
        "uint" => Token::TypeKeyword(Type::UInt),
        "boolean" => Token::TypeKeyword(Type::Boolean),
        "true" => Token::TrueKeyword,
        "false" => Token::FalseKeyword,
        "function" => Token::FunctionKeyword,
        "infer" => Token::InferKeyword,
        "if" => Token::IfKeyword,
        "else" => Token::ElseKeyword,
        "return" => Token::ReturnKeyword,
        _ => Token::Identifier(value.to_owned()),
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
            vec![
                Token::TypeKeyword(Type::UInt),
                Token::TypeKeyword(Type::Boolean),
                Token::TrueKeyword,
                Token::FalseKeyword,
                Token::FunctionKeyword,
                Token::InferKeyword,
                Token::IfKeyword,
                Token::ElseKeyword,
                Token::ReturnKeyword
            ]
        );
    }
}
