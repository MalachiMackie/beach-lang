mod ast;
mod cli;
mod evaluation;
mod parsing;
mod token_stream;
mod type_checking;

use std::env::args;

use cli::match_command;

fn main() {
    match_command(args());
}
