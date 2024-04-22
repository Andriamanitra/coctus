use crate::clash::TestCase;
use crate::formatter::show_whitespace;
use crate::outputstyle::OutputStyle;

#[derive(Debug, PartialEq, Clone)]
pub enum TestRunResult {
    Success,
    WrongOutput { stdout: String, stderr: String },
    RuntimeError { stdout: String, stderr: String },
    Timeout { stdout: String, stderr: String },
}

#[derive(Clone)]
pub struct TestRun<'a> {
    testcase: &'a TestCase,
    result: TestRunResult,
}

impl<'a> TestRun<'a> {
    pub fn new(testcase: &'a TestCase, result: TestRunResult) -> Self {
        Self { testcase, result }
    }

    pub fn expected(&self) -> &String {
        &self.testcase.test_out
    }

    pub fn actual(&self) -> &String {
        match &self.result {
            TestRunResult::Success => self.expected(),
            TestRunResult::RuntimeError { stdout, .. } => stdout,
            TestRunResult::WrongOutput { stdout, .. } => stdout,
            TestRunResult::Timeout { stdout, .. } => stdout,
        }
    }

    pub fn is_successful(&self) -> bool {
        self.result == TestRunResult::Success
    }

    pub fn print_result(&self, style: &OutputStyle) {
        let title = self.testcase.styled_title(style);
        match &self.result {
            TestRunResult::Success => {
                println!("{} {}", style.success.paint("PASS"), title);
            }

            TestRunResult::WrongOutput { stdout, stderr } => {
                println!("{} {}", style.failure.paint("FAIL"), title);
                print_failure(self.testcase, stdout, stderr, style);
            }

            TestRunResult::RuntimeError { stdout, stderr } => {
                println!("{} {}", style.error.paint("ERROR"), title);
                print_failure(self.testcase, stdout, stderr, style);
            }

            TestRunResult::Timeout { stdout, stderr } => {
                println!("{} {}", style.error.paint("TIMEOUT"), title);
                print_failure(self.testcase, stdout, stderr, style);
            }
        }
    }
}

fn print_failure(testcase: &TestCase, stdout: &str, stderr: &str, ostyle: &OutputStyle) {
    println!(
        "{}\n{}\n{}\n{}",
        ostyle.secondary_title.paint("===== INPUT ======"),
        testcase.styled_input(ostyle),
        ostyle.secondary_title.paint("==== EXPECTED ===="),
        testcase.styled_output(ostyle)
    );

    println!("{}", &ostyle.secondary_title.paint("===== STDOUT ====="));
    print_diff(testcase, stdout, ostyle);

    if !stderr.is_empty() {
        println!(
            "{}\n{}",
            ostyle.secondary_title.paint("===== STDERR ====="),
            ostyle.stderr.paint(stderr.trim_end())
        );
    }
}

// https://stackoverflow.com/a/40457615/5465108
struct LinesWithEndings<'a> {
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
            return None
        }
        let split = self.input.find('\n').map(|i| i + 1).unwrap_or(self.input.len());
        let (line, rest) = self.input.split_at(split);
        self.input = rest;
        Some(line)
    }
}

fn print_diff(testcase: &TestCase, stdout: &str, ostyle: &OutputStyle) {
    use dissimilar::Chunk::*;
    use itertools::EitherOrBoth::{Both, Left, Right};
    use itertools::Itertools;

    let diff_red = &ostyle.diff_red;
    let diff_ws_red = &ostyle.diff_red_whitespace;
    let diff_green = &ostyle.diff_green;
    let diff_ws_green = &ostyle.diff_green_whitespace;

    if stdout.is_empty() {
        println!("{}", ostyle.dim_color.paint("(no output)"));
        return
    }

    let expected_lines = LinesWithEndings::from(&testcase.test_out);
    let actual_lines = LinesWithEndings::from(stdout);

    let mut missing_lines = 0;
    for either_or_both in expected_lines.zip_longest(actual_lines) {
        match either_or_both {
            Left(_) => missing_lines += 1,
            Right(s) => print!("{}", show_whitespace(s, diff_red, diff_ws_red)),
            Both(a, b) => {
                let mut prev_deleted = false;

                for chunk in dissimilar::diff(a, b) {
                    match chunk {
                        Equal(text) if prev_deleted => {
                            let mut chars = text.chars();
                            let first_char = chars.next().expect("no chars???").to_string();
                            let rest = chars.as_str();
                            print!("{}", show_whitespace(&first_char, diff_red, diff_ws_red));
                            if !rest.is_empty() {
                                print!("{}", show_whitespace(rest, diff_green, diff_ws_green));
                            }
                        }
                        Equal(text) => print!("{}", show_whitespace(text, diff_green, diff_ws_green)),
                        Insert(text) => print!("{}", show_whitespace(text, diff_red, diff_ws_red)),
                        Delete(_) => {}
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
        println!("{}", ostyle.dim_color.paint(msg));
    }
}
