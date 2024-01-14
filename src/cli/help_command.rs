use crate::cli::get_commands;

use super::BeachCommand;

pub struct HelpCommand;

impl BeachCommand for HelpCommand {
    fn name(&self) -> &'static str {
        "help"
    }

    fn usage(&self) -> &'static str {
        "beach help"
    }

    fn description(&self) -> &'static str {
        "prints help information for the beach cli"
    }

    fn run(&self, _: Vec<String>) -> Result<(), String> {
        let commands = get_commands();
        println!(
            "usage: beach [command] [command_args]\n\t{}",
            commands
                .into_iter()
                .map(|command| format!("{}\t{}", command.name(), command.description()))
                .collect::<Box<[String]>>()
                .join("\n\t")
        );

        Ok(())
    }
}
