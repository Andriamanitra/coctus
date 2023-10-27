use crate::clash::ClashTestCase;
use crate::formatter::Formatter;
use crate::outputstyle::TestCaseStyle;
use ansi_term::Color;
use anyhow::{anyhow, Result};
use std::process::Command;

#[derive(Debug, PartialEq)]
pub enum TestRunResult {
    Success,
    WrongOutput { stdout: String, stderr: String },
    RuntimeError { stdout: String, stderr: String },
}

pub fn make_command(cmd_str: &str) -> Result<Command> {
    match shlex::split(cmd_str) {
        Some(shlexed_cmd) if shlexed_cmd.is_empty() => Err(anyhow!("COMMAND is required")),
        Some(shlexed_cmd) => {
            let exe = &shlexed_cmd[0];
            let exe_args = &shlexed_cmd[1..];
            let mut cmd = Command::new(exe);
            cmd.args(exe_args);
            Ok(cmd)
        }
        _ => Err(anyhow!("Invalid COMMAND")),
    }
}

pub fn run_test(run: &mut Command, testcase: &ClashTestCase) -> Result<TestRunResult> {
    let mut run = run
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    let mut stdin = run.stdin.as_mut().unwrap();
    std::io::Write::write(&mut stdin, testcase.test_in.as_bytes())?;

    let output = run.wait_with_output()?;
    let stdout = String::from_utf8(output.stdout)?;
    let mut stdout = stdout.trim_end().to_string();
    // Fixes Windows "\r\n" endings not returning Success when they should
    stdout = stdout.replace("\r", "");
    let stderr = String::from_utf8(output.stderr)?;

    if stdout == testcase.test_out.trim_end() {
        Ok(TestRunResult::Success)
    } else if output.status.success() {
        Ok(TestRunResult::WrongOutput { stdout, stderr })
    } else {
        Ok(TestRunResult::RuntimeError { stdout, stderr })
    }
}

pub fn show_test_result(result: &TestRunResult, testcase: &ClashTestCase) {
    let testcase_style = TestCaseStyle::default();
    let formatter = Formatter::default();
    let title = testcase_style.title.paint(&testcase.title);
    match result {
        TestRunResult::Success => {
            println!("{} {}", testcase_style.success.paint("PASS"), title);
        }

        TestRunResult::WrongOutput { stderr, stdout } => {
            println!("{} {}", testcase_style.failure.paint("FAIL"), title);
            if !stderr.is_empty() {
                println!("{}", testcase_style.stderr.paint(stderr.trim_end()));
            }
            
            // print the original failed testcase?
            let help_message = format!(">>> The input was:\n>>> {}", &testcase.test_in);
            println!("{}", Color::Green.paint(&help_message));
            // compare in bulk
            print_pair(&stdout, &testcase.test_out, &formatter, &testcase_style);
            // compare line by line
            compare_line_by_line(&stdout, &testcase.test_out, &formatter, &testcase_style);
        }

        TestRunResult::RuntimeError { stdout, stderr } => {
            println!("{} {}", testcase_style.error.paint("ERROR"), title);
            println!("{}\n", testcase_style.stderr.paint(stderr.trim_end()));
            print_pair(&stdout, &testcase.test_out, &formatter, &testcase_style);
        }
    }
}

fn compare_line_by_line(stdout: &str, test_output: &str, formatter: &Formatter, testcase_style: &TestCaseStyle) {
    // If no output at all, special message
    if stdout.is_empty() && !test_output.is_empty() {
        if let Some(first_line) = test_output.lines().next() {
            print_pair(first_line, "No output", &formatter, testcase_style);
            return;
        } else {
            // Unreachable (means the test_out is empty), should catch
        }
    }

    let stdout_lines = stdout.lines();
    let testcase_lines = test_output.lines();
    for (idx_row, (actual_line, expected_line)) in stdout_lines.zip(testcase_lines).enumerate() {
        let (difference_str, idx_col) = compare_line(&actual_line, &expected_line, &formatter, &testcase_style);
        // Found an difference?
        if difference_str.len() > 0 {
            // Lines should be 1-indexed
            let position = format!("At line {}, char {}", idx_row + 1, idx_col);
            let cposition = testcase_style.failure.paint(&position).to_string();
            println!("{}", cposition);
            print_pair(&difference_str, expected_line, &formatter, &testcase_style);
            return;
        } 
    }

    // Didn't find any difference? There are less lines than expected
    // TODO: fix the repetition
    let idx_row = stdout.lines().count() - 1;
    // Do I really need to send the char when we know it's 0?
    let position = format!("At line {}, char {}", idx_row + 1, 0);
    let cposition = testcase_style.failure.paint(&position).to_string();
    println!("{}", cposition);
    let tmp: Vec<&str> = test_output.lines().collect();
    print_pair("Nothing", tmp[idx_row], &formatter, &testcase_style);
}

fn compare_line(actual: &str, expected: &str, formatter: &Formatter, testcase_style: &TestCaseStyle) -> (String, i32) {
    // If actual is empty, print a special message.
    if actual.len() == 0 && expected.len() > 0 {
        return ("Nothing".to_string(), 0);
    }

    let mut buffer = String::new();
    let mut error_count = 0;
    let mut idx_first_error: i32 = -1;

    for (idx, (c1, c2)) in actual.chars().zip(expected.chars()).enumerate() {
        if c1 == c2 {
            buffer += &formatter.show_whitespace(&c1.to_string(), &testcase_style.out, &testcase_style.whitespace);
        } else {
            buffer += &testcase_style.failure.paint(c1.to_string()).to_string();
            if idx_first_error == -1 {
                idx_first_error = idx as i32;
            }
            error_count += 1;
        }

        // To many errors, stop here
        if error_count > 5 {
            let difference_str = format!("{}...", buffer);
            return (difference_str, idx_first_error)
        }
    }
    if error_count > 0 {
        (buffer, idx_first_error) 
    } else {
        // Strings are the same, return this default.
        ("".to_string(), -1)
    }
}

fn print_pair(actual: &str, expected: &str, formatter: &Formatter, testcase_style: &TestCaseStyle) {
    println!(
        "==== EXPECTED ====\n{}",
        formatter.show_whitespace(expected, &testcase_style.out, &testcase_style.whitespace)
    );
    println!(
        "===== ACTUAL =====\n{}",
        formatter.show_whitespace(actual, &testcase_style.out, &testcase_style.whitespace)
    );  
}