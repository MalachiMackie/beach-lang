mod ast;
mod cli;
mod evaluation;
mod parsing;
mod token_stream;
mod type_checking;

use std::env::args;

use cli::match_command;

fn main() {
    if let Err(error) = match_command(args().skip(1).collect()) {
        println!("{error}");
    }
}
