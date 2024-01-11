mod ast;
mod evaluation;
mod parsing;
mod token_stream;
mod type_checking;

use ast::builders::ast_builder::AstBuilder;
use parsing::parse_program;

fn main() {
    let tokens = match parse_program(FIBONACCI_CODE) {
        Ok(tokens) => tokens,
        Err(errors) => {
            for error in errors {
                println!("{error}");
            }

            return;
        }
    };

    let ast = match AstBuilder::from_token_stream(tokens).map(|ast_builder| ast_builder.build()) {
        Err(errors) => {
            for error in errors {
                println!("{}", error.message);
            }

            return;
        }
        Ok(ast) => ast,
    };

    if let Err(errors) = ast.type_check() {
        for error in errors {
            println!("{}", error.message);
        }

        return;
    }

    ast.evaluate();
}

const FIBONACCI_CODE: &str = "
function fibonnacci(uint lower, uint higher, uint limit) -> uint
{
    infer next = lower + higher;
    if (next > limit)
    {
        return next;
    }

    print(next);

    return fibonnacci(higher, next, limit);
}

print(0);
print(1);
fibonnacci(0, 1, 10000);
";
