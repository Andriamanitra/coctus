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
    let stdout = stdout.trim_end().to_string();
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
            println!(
                "==== EXPECTED ====\n{}",
                formatter.show_whitespace(&testcase.test_out, &out_style, &ws_style)
            );
            println!(
                "===== ACTUAL =====\n{}\n",
                formatter.show_whitespace(stdout, &out_style, &ws_style)
            );
        }

        TestRunResult::RuntimeError { stderr } => {
            println!("{} {}", error_style.paint("ERROR"), title);
            println!("{}\n", stderr_style.paint(stderr.trim_end()));
        }
    }
}
