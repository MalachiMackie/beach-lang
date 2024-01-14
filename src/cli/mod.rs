mod help_command;

use std::env::Args;

use self::help_command::HelpCommand;

pub trait Command {
    fn name(&self) -> &'static str;
    fn usage(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn run(&self, args: Vec<String>);
}



fn get_commands() -> Box<[Box<dyn Command>]> {
    let commands: Vec<Box<dyn Command>> = vec![Box::new(HelpCommand)];

    commands.into_boxed_slice()
}

pub fn match_command(args: Args) {
    let mut args = args.skip(1);
    let commands = get_commands();

    let help_command = Box::new(HelpCommand) as Box<dyn Command>;

    let found_command = args
        .next()
        .map(|command_name| {
            commands
                .into_iter()
                .filter(|command| command.name().eq_ignore_ascii_case(&command_name))
                .next()
                .unwrap_or(&help_command)
        })
        .unwrap_or(&help_command);

    found_command.run(args.collect());
}
