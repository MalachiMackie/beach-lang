use std::fs;

use crate::{ast::builders::ast_builder::AstBuilder, parsing::parse_program};

use super::Command;

pub(super) struct RunCommand;

impl Command for RunCommand {
    fn name(&self) -> &'static str {
        "run"
    }

    fn usage(&self) -> &'static str {
        "beach run [program].bch"
    }

    fn description(&self) -> &'static str {
        "run a beach program"
    }

    fn run(&self, args: Vec<String>) -> Result<(), String> {
        let Some(program_file) = args.first() else {
            return Err(format!("usage {}", self.usage()));
        };

        if !program_file.ends_with(".bch") {
            return Err(format!("a beach program file must have .bch extension"));
        }

        if args.len() > 1 {
            println!(
                "the run command does not take any more sub commands or options\nusage: {}",
                self.usage()
            )
        }

        let code = match fs::read_to_string(program_file) {
            Err(err) => {
                return Err(format!("{:?}", err));
            }
            Ok(code) => code,
        };

        run(&code).map_err(|err| format!("Failed to run beach program: {}", err.join("\n")))
    }
}

fn run(code: &str) -> Result<(), Vec<String>> {
    let tokens = parse_program(code)?;

    let ast = AstBuilder::from_token_stream(tokens)
        .map_err(|errors| {
            errors
                .into_iter()
                .map(|err| err.message)
                .collect::<Vec<_>>()
        })?
        .build();

    ast.type_check().map_err(|errors| {
        errors
            .into_iter()
            .map(|err| err.message)
            .collect::<Vec<_>>()
    })?;

    ast.evaluate();

    Ok(())
}
