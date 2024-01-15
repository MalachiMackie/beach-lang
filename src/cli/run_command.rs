use std::fs;

use crate::{ast::builders::ast_builder::AstBuilder, parsing::parse_program};

use super::BeachCommand;

pub(super) struct RunCommand;

impl BeachCommand for RunCommand {
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
            return Err(format!("usage: {}", self.usage()));
        };

        if !program_file.ends_with(".bch") {
            return Err("a beach program file must have .bch extension".to_owned());
        }

        if args.len() > 1 {
            return Err(format!(
                "the run command does not take any more sub commands or options\nusage: {}",
                self.usage()
            ));
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
    let tokens = parse_program(code)?.into_iter().map(|(token, _)| token).collect();

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

#[cfg(test)]
mod tests {
    use crate::cli::BeachCommand;

    use super::RunCommand;

    #[test]
    fn run_command_name() {
        let command = RunCommand;

        assert_eq!(command.name(), "run");
    }

    #[test]
    fn run_command_description() {
        let command = RunCommand;

        assert_eq!(command.description(), "run a beach program")
    }

    #[test]
    fn run_command_usage() {
        let command = RunCommand;

        assert_eq!(command.usage(), "beach run [program].bch");
    }

    mod command_run {
        use crate::cli::{run_command::RunCommand, BeachCommand};

        #[test]
        fn empty_args() {
            let command = RunCommand;
            let args = Vec::new();

            let result = command.run(args);

            assert!(matches!(result, Err(error) if error == "usage: beach run [program].bch"));
        }

        #[test]
        fn too_many_args() {
            let command = RunCommand;

            let args = vec!["hello.bch".to_owned(), "somethingElse".to_owned()];

            let result = command.run(args);

            assert!(
                matches!(result, Err(error) if error == "the run command does not take any more sub commands or options\nusage: beach run [program].bch")
            )
        }

        #[test]
        fn incorrect_file_extension() {
            let command = RunCommand;

            let args = vec!["hello.rs".to_owned()];

            let result = command.run(args);

            assert!(
                matches!(result, Err(error) if error == "a beach program file must have .bch extension")
            )
        }
    }

    mod run_function {
        use crate::cli::run_command::run;

        #[test]
        fn parsing_error() {
            let code = "~";

            let result = run(code);

            assert!(matches!(result, Err(e) if e.len() == 1 && e[0] == "Unexpected character `~`"))
        }

        #[test]
        fn token_stream_error() {
            let code = "infer my_var = true";

            let result = run(code);

            assert!(matches!(result, Err(e) if e.len() == 1 && e[0] == "expected ;"))
        }

        #[test]
        fn type_checking_error() {
            let code = "if (1) {}";

            let result = run(code);

            assert!(
                matches!(result, Err(e) if e.len() == 1 && e[0] == "Expected type to be Boolean, but found UInt")
            )
        }

        #[test]
        fn run_ok() {
            let code = "if (true) { print(1); }";

            let result = run(code);

            assert!(result.is_ok());
        }
    }
}
