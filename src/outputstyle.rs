use ansi_term::{Color, Style};

pub struct OutputStyle {
    pub title: Style,
    pub link: Style,
    pub variable: Option<Style>,
    pub constant: Option<Style>,
    pub bold: Option<Style>,
    pub monospace: Option<Style>,
    pub input_example: Style,
    pub input_testcase: Style,
    pub input_whitespace: Option<Style>,
    pub output_example: Style,
    pub output_testcase: Style,
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
            link: Style::default(),
            variable: Some(Style::default()),
            constant: Some(Style::default()),
            bold: Some(Style::default()),
            monospace: Some(Style::default()),
            input_example: Style::default(),
            input_testcase: Style::default(),
            input_whitespace: None,
            output_example: Style::default(),
            output_testcase: Style::default(),
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
            link: Style::new().fg(Color::Yellow),
            variable: Some(Style::new().fg(Color::Yellow)),
            constant: Some(Style::new().fg(Color::Blue)),
            bold: Some(Style::new().italic()),
            monospace: Some(Style::default()),
            input_example: Style::new().fg(Color::White),
            input_testcase: Style::new().fg(Color::White),
            input_whitespace: Some(Style::new().fg(Color::Black).dimmed()),
            output_example: Style::new().fg(Color::Green),
            output_testcase: Style::new().fg(Color::White),
            output_whitespace: Some(Style::new().fg(Color::RGB(43, 43, 43))),
            // output_whitespace: Some(Style::new().fg(Color::White).dimmed()),
            success: Style::new().on(Color::Green),
            failure: Style::new().on(Color::Red),
            error:  Style::new().on(Color::Red),
            stderr: Style::new().fg(Color::Red),
        }
    }
}
