use crate::clash::ClashTestCase;
use crate::formatter;
use crate::outputstyle::OutputStyle;
use anyhow::{anyhow, Result};
use std::{process::Command, cmp};
use difference::Changeset;

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
    let stdout = stdout.replace("\r\n", "\n").trim_end().to_string();
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
    let ostyle = &OutputStyle::default();

    let title = ostyle.title.paint(&testcase.title);
    match result {
        TestRunResult::Success => {
            println!("{} {}", ostyle.success.paint("PASS"), title);
        }

        TestRunResult::WrongOutput { stderr, stdout } => {
            println!("{} {}", ostyle.failure.paint("FAIL"), title);
            print_diff(testcase, stdout, ostyle);
            if !stderr.is_empty() {
                println!("{}", ostyle.stderr.paint(stderr.trim_end()));
            }

            let fmt_stdout = zipped_difference(testcase, stdout, &ostyle);
            print_diff(testcase, &fmt_stdout, ostyle);

            // using difference crate
            let plain = OutputStyle::plain();
            print_linewise_difference(testcase, stdout, &plain);
            print_block_difference(testcase, stdout, &plain);
        }

        TestRunResult::RuntimeError { stdout, stderr } => {
            println!("{} {}", ostyle.error.paint("ERROR"), title);
            if !stdout.is_empty() {
                print_diff(testcase, stdout, ostyle);
            }
            if !stderr.is_empty() {
                println!("{}", ostyle.stderr.paint(stderr.trim_end()));
            }
        }
    }
}

fn print_linewise_difference(testcase: &ClashTestCase, stdout: &str, ostyle: &OutputStyle) {
    let expected = &testcase.test_out;
    let received = &stdout;

    println!("\nLINEWISE DIFFERENCE ======\n");
    for line_number in 0..(cmp::max(expected.lines().count(), received.lines().count())) {
        let exp_line = expected.lines().nth(line_number).unwrap_or("");
        let rec_line = received.lines().nth(line_number).unwrap_or("");
        let diffed_line = format!("{}", Changeset::new(rec_line, exp_line, ""));
        println!("{}",
            if let Some(ws_style) = ostyle.output_whitespace {
                formatter::show_whitespace(&diffed_line, &ostyle.output, &ws_style)
            } else {
                ostyle.output.paint(&diffed_line).to_string()
            }
        )
    }
}

fn print_block_difference(testcase: &ClashTestCase, stdout: &str, ostyle: &OutputStyle) {
    let expected = &testcase.test_out;
    let received = &stdout;

    println!("\nBLOCK DIFFERENCE ======\n");
    let compared = format!("{}", Changeset::new(received, expected, ""));
    println!("{}",
        if let Some(ws_style) = ostyle.output_whitespace {
            formatter::show_whitespace(&compared, &ostyle.output, &ws_style)
        } else {
            ostyle.output.paint(&compared).to_string()
        }
    )
}

fn zipped_difference(testcase: &ClashTestCase, stdout: &str, ostyle: &OutputStyle) -> String {
    let expected = &testcase.test_out;
    let received = &stdout;
    let mut buffer = String::new();

    for (cexp, crec) in expected.chars().zip(received.chars()) {
        if crec == cexp {
            if crec == '\n' {
                buffer += &crec.to_string()
            } else {
                let tmp = if let Some(ws_style) = ostyle.output_whitespace {
                    formatter::show_whitespace(&crec.to_string(), &ostyle.output, &ws_style)
                } else {
                    ostyle.output.paint(&crec.to_string()).to_string()
                };
                buffer += &tmp;
            };
        } else {
            if crec == '\n' {
                buffer += &ostyle.failure.paint("¶\n").to_string()
            } else {
                let tmp = if let Some(ws_style) = ostyle.output_whitespace {
                    formatter::show_whitespace(&crec.to_string(), &ostyle.output, &ws_style)
                } else {
                    ostyle.output.paint(&crec.to_string()).to_string()
                };
                buffer += &ostyle.failure.paint(tmp.to_string()).to_string()
            };
        }
    }
    // There's more expected
    if received.chars().count() < expected.chars().count() {
        return buffer;
    }
    // There's more received 
    for irec in expected.chars().count()..received.chars().count() {
        let crec = received.chars().nth(irec).unwrap();
        buffer += &ostyle.failure.paint(crec.to_string()).to_string();
    }

    buffer
}

pub fn print_diff(testcase: &ClashTestCase, stdout: &str, ostyle: &OutputStyle) {
    println!(
        "{}\n{}",
        &ostyle.secondary_title.paint("===== INPUT ======"),
        testcase.styled_input(ostyle)
    );
    println!(
        "{}\n{}",
        &ostyle.secondary_title.paint("==== EXPECTED ===="),
        testcase.styled_output(ostyle)
    );
    println!(
        "{}\n{}",
        &ostyle.secondary_title.paint("==== RECEIVED ===="),
        if let Some(ws_style) = ostyle.output_whitespace {
            formatter::show_whitespace(stdout, &ostyle.output, &ws_style)
        } else {
            ostyle.output.paint(stdout).to_string()
        }
    );
    println!("{}", ostyle.secondary_title.paint("=================="));
}
