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
        }
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
            monospace: Style::default(),
            input: Style::new().fg(Color::White),
            input_whitespace: Some(Style::new().fg(Color::RGB(43, 43, 43))),
            output: Style::new().fg(Color::White),
            output_whitespace: Some(Style::new().fg(Color::RGB(43, 43, 43))),
            success: Style::new().on(Color::Green),
            failure: Style::new().on(Color::Red),
            error: Style::new().on(Color::Red),
            stderr: Style::new().fg(Color::Red),
        }
    }
}
