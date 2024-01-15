mod ast;
mod cli;
mod evaluation;
mod parsing;
mod token_stream;
mod type_checking;

use std::{env::args, process::exit};

use cli::match_command;

fn main() {
    if let Ok(_) = match_command(args().skip(1).collect()) {
        exit(0);
    } else {
        exit(1);
    }
}
