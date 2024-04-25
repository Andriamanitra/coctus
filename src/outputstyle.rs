use ansi_term::{Color, Style};

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
