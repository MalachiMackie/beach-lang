use crate::cli::get_commands;

use super::Command;

pub struct HelpCommand;

impl Command for HelpCommand {
    fn name(&self) -> &'static str {
        "help"
    }

    fn usage(&self) -> &'static str {
        "beach help"
    }

    fn description(&self) -> &'static str {
        "prints help information for the beach cli"
    }

    fn run(&self, _: Vec<String>) {
        let commands = get_commands();
        println!(
            "usage: beach [command] [command_args]\n\t{}",
            commands
                .into_iter()
                .map(|command| format!("{}\t{}", command.name(), command.description()))
                .collect::<Box<[String]>>()
                .join("\n\t")
        )
    }
}
