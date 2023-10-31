use crate::clash::TestCase as ClashTestCase;
use crate::formatter::show_whitespace;
use crate::outputstyle::OutputStyle;
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
            print_testcase(testcase, stdout, ostyle);
            print_diff(testcase, stdout, ostyle);
            if !stderr.is_empty() {
                println!("{}", ostyle.stderr.paint(stderr.trim_end()));
            }
        }

        TestRunResult::RuntimeError { stdout, stderr } => {
            println!("{} {}", ostyle.error.paint("ERROR"), title);
            if !stdout.is_empty() {
                print_testcase(testcase, stdout, ostyle);
                print_diff(testcase, stdout, ostyle);
            }
            if !stderr.is_empty() {
                println!("{}", ostyle.stderr.paint(stderr.trim_end()));
            }
        }
    }
}

pub fn print_testcase(testcase: &ClashTestCase, stdout: &str, ostyle: &OutputStyle) {
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
            show_whitespace(stdout, &ostyle.output, &ws_style)
        } else {
            ostyle.output.paint(stdout).to_string()
        }
    );
    println!("{}", ostyle.secondary_title.paint("=================="));
}

// https://stackoverflow.com/a/40457615/5465108
pub struct LinesWithEndings<'a> {
    input: &'a str,
}

impl<'a> LinesWithEndings<'a> {
    pub fn from(input: &'a str) -> LinesWithEndings<'a> {
        LinesWithEndings { input }
    }
}

impl<'a> Iterator for LinesWithEndings<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<&'a str> {
        if self.input.is_empty() {
            return None;
        }
        let split = self.input.find('\n').map(|i| i + 1).unwrap_or(self.input.len());
        let (line, rest) = self.input.split_at(split);
        self.input = rest;
        Some(line)
    }
}

pub fn print_diff(testcase: &ClashTestCase, stdout: &str, ostyle: &OutputStyle) {
    use dissimilar::Chunk::*;
    use itertools::Itertools;
    use itertools::EitherOrBoth::{Left, Right, Both};

    // (TODO) temporary styling, to be replaced with OutputStyle eventually
    let green = ansi_term::Style::new().fg(ansi_term::Color::RGB(0,185,0));
    let red = ansi_term::Style::new().fg(ansi_term::Color::Red);
    let error_red = ansi_term::Style::new().fg(ansi_term::Color::Red).on(ansi_term::Color::RGB(70,0,0));
    let dim_color = ansi_term::Style::new().fg(ansi_term::Color::RGB(50,50,50));
    let ws_style = &ostyle.output_whitespace.unwrap_or(ostyle.output);

    if stdout.is_empty() {
        println!("{}", dim_color.paint("(no output)"));
        return
    }

    let expected_lines = LinesWithEndings::from(&testcase.test_out);
    let actual_lines = LinesWithEndings::from(stdout);

    let mut missing_lines = 0;
    for either_or_both in expected_lines.zip_longest(actual_lines) {
        match either_or_both {
            Left(_) => missing_lines += 1,
            Right(s) => print!("{}", show_whitespace(s, &red, &error_red)),
            Both(a, b) => {
                let mut prev_deleted = false;

                for chunk in dissimilar::diff(a, b) {
                    match chunk {
                        Equal(text) if prev_deleted => {
                            let mut chars = text.chars();
                            let first_char = chars.next().expect("no chars???").to_string();
                            let rest = chars.as_str();
                            print!("{}", show_whitespace(&first_char, &red, &error_red));
                            if !rest.is_empty() {
                                print!("{}", show_whitespace(rest, &green, ws_style));
                            }
                        },
                        Equal(text) => print!("{}", show_whitespace(text, &green, ws_style)),
                        Insert(text) => print!("{}", show_whitespace(text, &red, &error_red)),
                        Delete(_) => {},
                    }

                    prev_deleted = matches!(chunk, Delete(_));
                }
            }
        }
    }

    if !stdout.ends_with('\n') {
        println!()
    }

    if missing_lines > 0 {
        let msg = format!("(expected {} more lines)", missing_lines);
        println!("{}", dim_color.paint(msg));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    fn test_diff(expected: &str, actual: &str) {
        let ostyle = &OutputStyle::default();
        let testcase = ClashTestCase {
            title: String::from("diff test"),
            test_in: String::from("some test input"),
            test_out: expected.to_string(),
            is_validator: false,
        };

        print_diff(&testcase, actual, ostyle);
    }

    #[test]
    fn diff_single_number() {
        test_diff("66", "69");
    }

    #[test]
    fn diff_missing_stuff_at_end_of_line() {
        test_diff(
            indoc! {r#"
                ####
                   #
                  ###
                 #####
                #######
                #######
                 #####
                  ###
                   #"#
            },
            indoc! {r#"
                ####
                   #
                  ###
                 #####
                #####
                #######
                 #####"#
            }
        );
    }

    #[test]
    fn diff_one_wrong_line() {
        test_diff(
            indoc! {r#"
                true
                true
                false
                false
                true"#
            },
            indoc! {r#"
                true
                true
                true
                false
                true"#
            }
        )
    }

    #[test]
    fn diff_missing_chars_in_the_middle() {
        test_diff(
            indoc! {r#"
                50.0% Apple
                33.3% Banana
                16.7% Pear"#
            },
            indoc! {r#"
                50.0 Apple
                33.3 Banana
                16.7 Pear"#
            }
        )
    }

    #[test]
    fn diff_annoyingly_long() {
        test_diff(
            "++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++.+++++++++++++++++++++++++++++.+++++++..+++.-------------------------------------------------------------------------------.+++++++++++++++++++++++++++++++++++++++++++++++++++++++.++++++++++++++++++++++++.+++.------.--------.",
            "++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++.+++++++++++++++++++++++++++++.+++++++..+++.------------------------------------------------------------------------------.+++++++++++++++++++++++++++++++++++++++++++++++++++++++.++++++++++++++++++++++++.+++.-----.-------."
        )
    }

    #[test]
    fn diff_extra_garbage() {
        test_diff(
            indoc! {r#"
                Team 1: Lisa, Ann
                Team 2: Frank, Helen
                Team 3: Lucy, Fran
            "#
            },
            indoc! {r#"
                Team 1: Lisa, Ann, Frank
                Team 2: Frank, Lucy, Helen
                Team 3: Lisa, Lucy, Fran
            "#
            }
        )
    }

    #[test]
    fn diff_spacing() {
        test_diff("(0.00,2.00),(2.00,6.00)", "(0.00, 2.00), (2.0, 6.0)")
    }

    #[test]
    fn diff_casing() {
        test_diff("true\ntrue\nfalse", "True\nFalse\nFalse")
    }

    #[test]
    fn diff_forgot_to_remove_debug_print() {
        test_diff("1337", "4 5\n1337")
    }

    #[test]
    fn diff_off_by_one() {
        test_diff("Keeping values between -9 and 21", "Keeping values between -10 and 20")
    }

    #[test]
    fn diff_unicorn() {
        test_diff(
            indoc! {r#"
                \
                 \
                  \
                   \
                  _oO^____
                 (._,     \
                    \  _\ /\
                     || ||
                 ~~~~~~~~~~~~~"#
            },
            indoc! {r#"
                \
                 \
                  \
                   \
                 _oO^____
                (._,     \
                   \  _\ /\
                    || ||
                ~~~~~~~~~~~~~"#
            }
        )
    }

    #[test]
    fn diff_empty_lines() {
        test_diff("much\noutput", "");
        test_diff("hello\nworld", "hello\n\nworld");
        test_diff("hello\n\nworld", "hello\nworld");
    }

    #[test]
    fn diff_trailing_whitespace() {
        test_diff("extra newline", "extra newline\n");
        test_diff("two newlines", "two newlines\n\n");
        test_diff("1 1 2 3 5", "1 1 2 3 5 ");
    }

    #[test]
    fn diff_stuff_missing_at_the_end() {
        test_diff("24 hours", "24 hour");
    }

    #[test]
    fn diff_multibyte_characters() {
        // just to make sure nothing crashes
        test_diff("漢字", "kanji");
        test_diff("kanji", "漢字");
        test_diff("漢X字", "漢字");
        test_diff("漢字", "漢X字");
        test_diff("Привет", "привет");
        test_diff("(╯°□°）╯︵ ┻━┻)", "(╯*益*）╯︵ ┻━┻)")
    }

    #[test]
    fn diff_words_at_the_end() {
        test_diff("hello world", "hello forld world");
    }

    #[test]
    fn diff_extra_newlines() {
        test_diff("abcd", "a\nb\n\nc\nd");
    }
}
