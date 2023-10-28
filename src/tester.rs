use crate::clash::ClashTestCase;
use crate::formatter::Formatter;
use crate::outputstyle::OutputStyle;
use ansi_term::{Color,Style};
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
    // Create class Tester?
    let style = OutputStyle::default();
    let formatter = Formatter::default();

    let title = style.title.paint(&testcase.title);
    match result {
        TestRunResult::Success => {
            println!("{} {}", style.success.paint("PASS"), title);
        }

        TestRunResult::WrongOutput { stderr, stdout } => {
            println!("{} {}", style.failure.paint("FAIL"), title);
            if !stderr.is_empty() {
                println!("{}", style.stderr.paint(stderr.trim_end()));
            }

            let input = &testcase.test_in;
            let expected = &testcase.test_out;
            let received = &stdout;
            // compare in bulk
            print_pair(input, expected, received, &formatter, &style);
            println!("");
            // compare but stop printing RECEIVED when an error is found (+format)
            println!("RAFA ======");
            compare(input, expected, received, &formatter, &style);
            // using difference crate
            let plain = OutputStyle::plain();
            print_linewise_difference(received, expected, &formatter, &plain);
            print_block_difference(received, expected, &formatter, &plain);
        }

        TestRunResult::RuntimeError { stdout, stderr } => {
            println!("{} {}", style.error.paint("ERROR"), title);
            println!("{}\n", style.stderr.paint(stderr.trim_end()));
            print_pair(&testcase.test_in,&testcase.test_out, &stdout, &formatter, &style);
        }
    }
}

fn print_linewise_difference(received: &str, expected: &str, formatter: &Formatter, style: &OutputStyle) {
    println!("\nLINEWISE DIFFERENCE ======\n");
    for line_number in 0..(cmp::max(expected.lines().count(), received.lines().count())) {
        let exp_line = expected.lines().nth(line_number).unwrap_or("");
        let rec_line = received.lines().nth(line_number).unwrap_or("");
        let diffed_line = format!("{}", Changeset::new(rec_line, exp_line, ""));
        println!("{}", formatter.show_whitespace(&diffed_line, &style.output_testcase, &style.output_whitespace))
    }
}

fn print_block_difference(received: &str, expected: &str, formatter: &Formatter, style: &OutputStyle) {
    println!("\nBLOCK DIFFERENCE ======\n");
    let compared = format!("{}", Changeset::new(received, expected, ""));
    println!("{}", formatter.show_whitespace(&compared, &style.output_testcase, &style.output_whitespace))
}

fn compare(input: &str, expected: &str, received: &str, formatter: &Formatter, style: &OutputStyle) {
    let errors_allowed = 1000; // This should turn off the abbreviation logic for the moment

    // If nothing was received at all, special message?
    if received.is_empty() {
        print_pair(input, expected, &Color::Red.paint("Nothing").to_string(), &formatter, style);
        return;
    }

    let mut buffer = String::new();
    let mut error_count = 0;

    for (cexp, crec) in expected.chars().zip(received.chars()) {
        if crec == cexp {
            if crec == '\n' {
                buffer += &crec.to_string()
            } else {
                buffer += &formatter.show_whitespace(&crec.to_string(), &style.output_testcase, &style.output_whitespace)
            };
        } else {
            if crec == '\n' {
                buffer += &style.failure.paint("¶\n").to_string()
            } else {
                buffer += &style.failure.paint(crec.to_string()).to_string()
            };
            error_count += 1;
        }
        // To many errors, stop here
        if error_count > errors_allowed {
            let fmt_received = format!("{}{}", buffer, &style.failure.paint("..."));
            print_pair(input, expected, &fmt_received, &formatter, &style);
            return;
        }
    }
    
    // There's more expected
    if received.chars().count() < expected.chars().count() {
        print_pair(input, expected, &buffer, &formatter, &style);
        return;
    }
    // There's more received 
    for irec in expected.chars().count()..received.chars().count() {
        let crec = received.chars().nth(irec).unwrap();
        buffer += &style.failure.paint(crec.to_string()).to_string();
        error_count += 1;
        // To many errors, stop here
        if error_count > errors_allowed {
            let fmt_received = format!("{}{}", buffer, &style.failure.paint("..."));
            print_pair(input, expected, &fmt_received, &formatter, &style);
            return;
        }
    }
    // There was more received, but we didn't reach the error threshold
    print_pair(input, expected, &buffer, &formatter, &style);
}

fn print_pair(input: &str, expected: &str, received: &str, formatter: &Formatter, style: &OutputStyle) {
    let title_style = Style::new().fg(Color::Purple).bold();
    println!(
        "{}\n{}",
        &title_style.paint("===== INPUT ======"),
        formatter.show_whitespace(&input, &style.output_testcase, &style.input_whitespace)
    );
    println!(
        "{}\n{}",
        &title_style.paint("==== EXPECTED ===="),
        formatter.show_whitespace(&expected, &style.output_testcase, &style.output_whitespace)
    );
    println!(
        "{}\n{}",
        &title_style.paint("==== RECEIVED ===="),
        formatter.show_whitespace(&received, &style.output_testcase, &style.output_whitespace)
    );  
}
