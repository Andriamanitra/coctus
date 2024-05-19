use ansi_term::{Color, Style};
use clashlib::clash::{Clash, Testcase};
use clashlib::solution::TestResult;

use super::formatter::show_whitespace;
use super::lines_with_endings::LinesWithEndings;
use crate::internal::formatter::format_cg;

pub struct OutputStyle {
    pub title: Style,
    pub secondary_title: Style,
    pub link: Style,
    pub variable: Style,
    pub constant: Style,
    pub bold: Style,
    pub monospace: Style,
    pub input: Style,
    pub input_whitespace: Option<Style>,
    pub output: Style,
    pub output_whitespace: Option<Style>,
    pub success: Style,
    pub failure: Style,
    pub error: Style,
    pub stderr: Style,
    pub dim_color: Style,
    pub diff_green: Style,
    pub diff_green_whitespace: Option<Style>,
    pub diff_red: Style,
    pub diff_red_whitespace: Option<Style>,
}

impl OutputStyle {
    pub fn plain() -> Self {
        OutputStyle {
            title: Style::default(),
            secondary_title: Style::default(),
            link: Style::default(),
            variable: Style::default(),
            constant: Style::default(),
            bold: Style::default(),
            monospace: Style::default(),
            input: Style::default(),
            input_whitespace: None,
            output: Style::default(),
            output_whitespace: None,
            success: Style::default(),
            failure: Style::default(),
            error: Style::default(),
            stderr: Style::default(),
            dim_color: Style::default(),
            diff_green: Style::default(),
            diff_green_whitespace: Some(Style::default()),
            diff_red: Style::default(),
            diff_red_whitespace: Some(Style::default()),
        }
    }
    pub fn from_env(show_whitespace: bool) -> Self {
        let mut ostyle = match std::env::var_os("NO_COLOR") {
            Some(s) if s.is_empty() => OutputStyle::default(),
            Some(_) => OutputStyle::plain(),
            None => OutputStyle::default(),
        };
        if show_whitespace {
            ostyle.input_whitespace = ostyle.input_whitespace.or(Some(ostyle.input));
            ostyle.output_whitespace = ostyle.output_whitespace.or(Some(ostyle.output));
            ostyle.diff_green_whitespace = ostyle.diff_green_whitespace.or(Some(ostyle.diff_green));
            ostyle.diff_red_whitespace = ostyle.diff_red_whitespace.or(Some(ostyle.diff_red));
        } else {
            ostyle.input_whitespace = None;
            ostyle.output_whitespace = None;
            ostyle.diff_green_whitespace = None;
            ostyle.diff_red_whitespace = None;
        }
        ostyle
    }
}

impl Default for OutputStyle {
    fn default() -> Self {
        OutputStyle {
            title: Style::new().fg(Color::Yellow).bold(),
            secondary_title: Style::new().fg(Color::Purple),
            link: Style::new().fg(Color::Yellow),
            variable: Style::new().fg(Color::Yellow),
            constant: Style::new().fg(Color::Blue),
            bold: Style::new().bold(),
            monospace: Style::new().on(Color::RGB(43, 43, 43)),
            input: Style::new().fg(Color::White),
            input_whitespace: Some(Style::new().fg(Color::RGB(43, 43, 43))),
            output: Style::new().fg(Color::White),
            output_whitespace: Some(Style::new().fg(Color::RGB(43, 43, 43))),
            success: Style::new().on(Color::Green),
            failure: Style::new().on(Color::Red),
            error: Style::new().on(Color::Red),
            stderr: Style::new().fg(Color::Red),
            dim_color: Style::new().fg(Color::RGB(50, 50, 50)),
            diff_green: Style::new().fg(Color::RGB(111, 255, 111)),
            diff_green_whitespace: Some(Style::new().fg(Color::RGB(0, 70, 0))),
            diff_red: Style::new().fg(Color::RGB(255, 111, 111)),
            diff_red_whitespace: Some(Style::new().fg(Color::Red).on(Color::RGB(70, 0, 0))),
        }
    }
}

impl OutputStyle {
    pub fn styled_testcase_title(&self, testcase: &Testcase) -> String {
        self.title.paint(format!("#{} {}", testcase.index, testcase.title)).to_string()
    }

    pub fn styled_testcase_input(&self, testcase: &Testcase) -> String {
        show_whitespace(&testcase.test_in, &self.input, &self.input_whitespace)
    }

    pub fn styled_testcase_output(&self, testcase: &Testcase) -> String {
        show_whitespace(&testcase.test_out, &self.output, &self.output_whitespace)
    }

    pub fn print_headers(&self, clash: &Clash) {
        println!("{}\n", self.title.paint(format!("=== {} ===", clash.title())));
        println!("{}\n", self.link.paint(clash.codingame_link()));
    }

    pub fn print_statement(&self, clash: &Clash) {
        println!("{}\n", format_cg(clash.statement(), self));
        println!("{}\n{}\n", self.title.paint("Input:"), format_cg(clash.input_description(), self));
        println!("{}\n{}\n", self.title.paint("Output:"), format_cg(clash.output_description(), self));
        if let Some(constraints) = clash.constraints() {
            println!("{}\n{}\n", self.title.paint("Constraints:"), format_cg(constraints, self));
        }

        let example = clash.testcases().first().expect("example puzzle should have at least one testcase");
        println!(
            "{}\n{}\n{}\n{}",
            self.title.paint("Example:"),
            self.styled_testcase_input(example),
            self.title.paint("Expected output:"),
            self.styled_testcase_output(example),
        );
    }

    pub fn print_testcases(&self, clash: &Clash, selection: Vec<usize>) {
        // Skips validators: -t 1 will print the example, -t 2 will print the second
        // test (skipping validator 1)
        for (idx, testcase) in clash.testcases().iter().filter(|t| !t.is_validator).enumerate() {
            if selection.contains(&idx) {
                println!(
                    "{}\n{}\n\n{}\n",
                    self.styled_testcase_title(testcase),
                    self.styled_testcase_input(testcase),
                    self.styled_testcase_output(testcase),
                );
            }
        }
    }

    pub fn print_reverse_mode(&self, clash: &Clash) {
        self.print_headers(clash);
        println!("{}\n", self.title.paint("REVERSE!"));
        let selection = (0..clash.testcases().len()).collect::<Vec<usize>>();
        self.print_testcases(clash, selection);
    }

    fn print_diff(&self, testcase: &Testcase, stdout: &str) {
        use dissimilar::Chunk::*;
        use itertools::EitherOrBoth::{Both, Left, Right};
        use itertools::Itertools;

        let diff_red = &self.diff_red;
        let diff_ws_red = &self.diff_red_whitespace;
        let diff_green = &self.diff_green;
        let diff_ws_green = &self.diff_green_whitespace;

        if stdout.is_empty() {
            println!("{}", self.dim_color.paint("(no output)"));
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
                                let first_char = chars.next().expect("diff chunk should not be empty");
                                let rest = chars.as_str();
                                print!("{}", show_whitespace(&first_char.to_string(), diff_red, diff_ws_red));
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
            println!("{}", self.dim_color.paint(msg));
        }
    }

    pub fn print_result(&self, testcase: &Testcase, test_result: &TestResult) {
        let title = self.styled_testcase_title(testcase);
        match test_result {
            TestResult::Success => {
                println!("{} {}", self.success.paint("PASS"), title);
            }

            TestResult::UnableToRun { error_msg } => {
                println!("{} {}", self.failure.paint("ERROR"), title);
                println!(" {}", self.stderr.paint(error_msg));
            }

            TestResult::WrongOutput { stdout, stderr } => {
                println!("{} {}", self.failure.paint("FAIL"), title);
                self.print_failure(testcase, stdout, stderr);
            }

            TestResult::RuntimeError { stdout, stderr } => {
                println!("{} {}", self.error.paint("ERROR"), title);
                self.print_failure(testcase, stdout, stderr);
            }

            TestResult::Timeout { stdout, stderr } => {
                println!("{} {}", self.error.paint("TIMEOUT"), title);
                self.print_failure(testcase, stdout, stderr);
            }
        }
    }

    fn print_failure(&self, testcase: &Testcase, stdout: &str, stderr: &str) {
        println!(
            "{}\n{}\n{}\n{}",
            self.secondary_title.paint("===== INPUT ======"),
            self.styled_testcase_input(testcase),
            self.secondary_title.paint("==== EXPECTED ===="),
            self.styled_testcase_output(testcase)
        );

        println!("{}", &self.secondary_title.paint("===== STDOUT ====="));
        self.print_diff(testcase, stdout);

        if !stderr.is_empty() {
            println!(
                "{}\n{}",
                self.secondary_title.paint("===== STDERR ====="),
                self.stderr.paint(stderr.trim_end())
            );
        }
    }
}
