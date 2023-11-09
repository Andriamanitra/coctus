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
            monospace: Style::new().on(Color::RGB(43, 43, 43)),
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

/// Returns a merged style. Not symmetric.
pub fn merge_styles(style: &Style, other: &Style) -> Style {
    let mut merged_style = style.clone();
    let def = Style::default();

    // If style has an attribute that is the same as the default one, 
    // then overwrite it with other's attribute.
    if style.foreground == def.foreground {
        merged_style.foreground = other.foreground
    }
    if style.background == def.background {
        merged_style.background = other.background
    }
    if style.is_bold == def.is_bold {
        merged_style.is_bold = other.is_bold
    }
    if style.is_italic == def.is_italic {
        merged_style.is_italic = other.is_italic
    }
    if style.is_underline == def.is_underline {
        merged_style.is_underline = other.is_underline
    }
    if style.is_blink == def.is_blink {
        merged_style.is_blink = other.is_blink
    }
    if style.is_reverse == def.is_reverse {
        merged_style.is_reverse = other.is_reverse
    }
    if style.is_hidden == def.is_hidden {
        merged_style.is_hidden = other.is_hidden
    }
    if style.is_strikethrough == def.is_strikethrough {
        merged_style.is_strikethrough = other.is_strikethrough
    }

    merged_style
}
