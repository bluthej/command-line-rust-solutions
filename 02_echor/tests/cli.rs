use assert_cmd::Command;
use predicates::prelude::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn dies_no_args() -> TestResult {
    let mut cmd = Command::cargo_bin("echor")?;
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
    Ok(())
}

#[test]
fn runs() -> TestResult {
    let mut cmd = Command::cargo_bin("echor")?;
    cmd.arg("hello").assert().success();
    Ok(())
}

fn run(args: &[&str], expected: String) -> TestResult {
    let mut cmd = Command::cargo_bin("echor")?;
    cmd.args(args).assert().success().stdout(expected);
    Ok(())
}

#[test]
fn hello1() -> TestResult {
    run(&["Hello there"], String::from("Hello there\n"))
}

#[test]
fn hello2() -> TestResult {
    run(&["Hello", "there"], String::from("Hello there\n"))
}

#[test]
fn hello1_no_newline() -> TestResult {
    run(&["Hello  there", "-n"], String::from("Hello  there"))
}

#[test]
fn hello2_no_newline() -> TestResult {
    run(&["-n", "Hello", "there"], String::from("Hello there"))
}
