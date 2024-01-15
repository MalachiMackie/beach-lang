mod help_command;
mod run_command;

use self::{help_command::HelpCommand, run_command::RunCommand};

pub trait BeachCommand {
    fn name(&self) -> &'static str;
    fn usage(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn run(&self, args: Vec<String>) -> Result<(), String>;
}

fn get_commands() -> Box<[Box<dyn BeachCommand>]> {
    let commands: Vec<Box<dyn BeachCommand>> = vec![Box::new(HelpCommand), Box::new(RunCommand)];

    commands.into_boxed_slice()
}

pub fn match_command(args: Vec<String>) -> Result<(), ()> {
    let mut args = args.into_iter();
    let commands = get_commands();

    let help_command = Box::new(HelpCommand) as Box<dyn BeachCommand>;

    let found_command = args.next().map(|command_name| {
        commands
            .into_iter()
            .filter(|command| command.name().eq_ignore_ascii_case(&command_name))
            .next()
    });

    let Some(Some(found_command)) = found_command else {
        _ = help_command.run(args.collect());
        return Err(());
    };

    if let Err(error) = found_command.run(args.collect()) {
        println!("{error}");
        Err(())
    } else {
        Ok(())
    }
}
