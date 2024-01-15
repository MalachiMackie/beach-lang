use std::process::Command;

#[test]
fn help_command_prints_correctly() {
    let mut command = Command::new("cargo");
    command.args(vec!["run", "help"]);
    let output_result = command.output();

    assert!(output_result.is_ok());
    let output = output_result.unwrap();

    assert!(output.status.success());

    let stdout = output.stdout;
    let expected: Vec<u8> = "usage: beach [command] [command_args]
\thelp\tprints help information for the beach cli
\trun\trun a beach program\n"
        .into();

    assert_eq!(stdout, expected);
}

#[test]
fn help_command_prints_when_no_args_are_provided() {
    let mut command = Command::new("cargo");
    command.args(vec!["run"]);

    let output_result = command.output();

    assert!(output_result.is_ok());
    let output = output_result.unwrap();

    assert!(!output.status.success());

    let stdout = output.stdout;
    let expected: Vec<u8> = "usage: beach [command] [command_args]
\thelp\tprints help information for the beach cli
\trun\trun a beach program\n"
        .into();

    assert_eq!(stdout, expected);
}
