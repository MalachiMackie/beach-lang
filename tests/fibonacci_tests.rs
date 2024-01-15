use std::process::Command;

#[test]
fn fibonacci_example_executes_successfull() {
    let mut command = Command::new("cargo");
    command.args(vec!["run", "run", "./examples/fibonacci.bch"]);

    let output_result = command.output();

    assert!(output_result.is_ok());
    let output = output_result.unwrap();

    assert!(output.status.success());

    let stdout = output.stdout;

    let expected: Vec<u8> = "0
1
1
2
3
5
8
13
21
34
55
89
144
233
377
610
987
1597
2584
4181
6765
"
    .into();

    assert_eq!(stdout, expected);
}
