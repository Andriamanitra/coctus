use crate::clash::ClashTestCase;
use crate::formatter::Formatter;
use ansi_term::{Color, Style};
use anyhow::{anyhow, Result};
use std::process::Command;

#[derive(Debug, PartialEq)]
pub enum TestRunResult {
    Success,
    WrongOutput { stdout: String, stderr: String },
    RuntimeError { stderr: String },
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
        Ok(TestRunResult::RuntimeError { stderr })
    }
}

pub fn show_test_result(result: &TestRunResult, testcase: &ClashTestCase) {
    // TODO: use OutputStyle
    let success_style = Style::new().on(Color::Green);
    let failure_style = Style::new().on(Color::Red);
    let error_style = Style::new().on(Color::Red);
    let title_style = Style::new().fg(Color::Yellow);
    let stderr_style = Style::new().fg(Color::Red);
    let out_style = Style::new().fg(Color::White);
    let ws_style = Some(Style::new().fg(Color::RGB(43, 43, 43)));

    let title = title_style.paint(&testcase.title);
    match result {
        TestRunResult::Success => {
            println!("{} {}", success_style.paint("PASS"), title);
        }

        TestRunResult::WrongOutput { stderr, stdout } => {
            println!("{} {}", failure_style.paint("FAIL"), title);
            if !stderr.is_empty() {
                println!("{}", stderr_style.paint(stderr.trim_end()));
            }
            let formatter = Formatter::default();

            // compare in bulk
            print_pair(&stdout, &testcase.test_out, &formatter, out_style, ws_style);

            // compare line by line
            // If no output at all, special message
            if stdout.is_empty() && !testcase.test_out.is_empty() {
                if let Some(first_line) = testcase.test_out.lines().next() {
                    print_pair(first_line, "No output", &formatter, out_style, ws_style);
                } else {
                    // Unreachable (means the test_out is empty), should catch
                }
            }

            let stdout_lines = stdout.lines();
            let testcase_lines = testcase.test_out.lines();
            for (idx_row, (actual_line, expected_line)) in stdout_lines.zip(testcase_lines).enumerate() {
                let (difference_str, idx_col) = compare_line(&actual_line, &expected_line, &formatter, out_style, ws_style, failure_style);
                // Found an difference?
                if difference_str.len() > 0 {
                    // Lines should be 1-indexed
                    let position = format!("At line {}, char {}", idx_row + 1, idx_col);
                    let cposition = failure_style.paint(&position).to_string();
                    println!("{}", cposition);
                    print_pair(&difference_str, expected_line, &formatter, out_style, ws_style);
                    break;
                } 
            }
        }

        TestRunResult::RuntimeError { stderr } => {
            println!("{} {}", error_style.paint("ERROR"), title);
            println!("{}\n", stderr_style.paint(stderr.trim_end()));
        }
    }
}

fn compare_line(
    actual: &str, expected: &str, formatter: &Formatter, out_style: Style, ws_style: Option<Style>, failure_style: Style
) -> (String, i32) {
    // If actual is empty, print a special message.
    if actual.len() == 0 && expected.len() > 0 {
        return ("Nothing".to_string(), 0);
    }

    let mut buffer = String::new();
    let mut error_count = 0;
    let mut idx_first_error: i32 = -1;

    for (idx, (c1, c2)) in actual.chars().zip(expected.chars()).enumerate() {
        if c1 == c2 {
            buffer += &formatter.show_whitespace(&c1.to_string(), &out_style, &ws_style);
        } else {
            buffer += &failure_style.paint(c1.to_string()).to_string();
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
        return (buffer, idx_first_error) 
    } else {
        // Strings are the same, return this default.
        ("".to_string(), -1)
    }
}

fn print_pair(actual: &str, expected: &str, formatter: &Formatter, out_style: Style, ws_style: Option<Style>) {
    println!(
        "==== EXPECTED ====\n{}",
        formatter.show_whitespace(expected, &out_style, &ws_style)
    );
    println!(
        "===== ACTUAL =====\n{}",
        formatter.show_whitespace(actual, &out_style, &ws_style)
    );  
}