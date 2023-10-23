use crate::outputstyle::OutputStyle;
use ansi_term::Style;
use regex::Regex;

pub struct Formatter {
    re_variable: Regex,
    re_constant: Regex,
    re_bold: Regex,
    re_monospace: Regex,
}

impl Default for Formatter {
    fn default() -> Self {
        Formatter {
            re_variable:  Regex::new(r"\[\[(.+?)\]\]").unwrap(),
            re_constant:  Regex::new(r"\{\{(.+?)\}\}").unwrap(),
            re_bold:      Regex::new(r"<<(.+?)>>").unwrap(),
            // Also capture the previous '\n' if any (`Monospace` rule)
            re_monospace: Regex::new(r"\n?`([^`]+)`").unwrap(),
        }
    }
}

impl Formatter {
    // TODO: finish support `Monospace` (Newline trimming)
    // For testing `Monospace`: 23214afcdb23616e230097d138bd872ea7c75
    // TODO: support nested formatting <<Next [[n]] lines:>>
    pub fn format(&self, text: &str, output_style: &OutputStyle) -> String {
        // Trim consecutive spaces (imitates html behaviour)
        // But only if it's not in a Monospace block (between backticks ``)
        let re_backtick = Regex::new(r"(`[^`]+`)|([^`]+)").unwrap();
        let re_spaces = Regex::new(r" +").unwrap();

        let mut result = re_backtick.replace_all(text, |caps: &regex::Captures| {
            if let Some(backtick_text) = caps.get(1) {
                backtick_text.as_str().to_string()
            } else if let Some(non_backtick_text) = caps.get(2) {
                re_spaces.replace_all(non_backtick_text.as_str(), " ").to_string()
            } else {
                "".to_string()
            }
        }).to_string();

        // Replace codingame formatting with proper colours
        if let Some(style) = output_style.variable {
            result = self.re_variable.replace_all(&result, |caps: &regex::Captures| {
                style.paint(&caps[1]).to_string()
            }).to_string();
        }
        if let Some(style) = output_style.constant {
            result = self.re_constant.replace_all(&result, |caps: &regex::Captures| {
                style.paint(&caps[1]).to_string()
            }).to_string();
        }
        if let Some(style) = output_style.bold {
            result = self.re_bold.replace_all(&result, |caps: &regex::Captures| {
                style.paint(&caps[1]).to_string()
            }).to_string();
        }
        if let Some(style) = output_style.monospace {
            result = self.re_monospace.replace_all(&result, |caps: &regex::Captures| {
                // Extra newline at the start for monospace
                format!("\n{}", style.paint(&caps[1]).to_string())
            }).to_string();
        }

        result
    }

    // For visibility: turn spaces into "•" and newlines into "¶"
    pub fn show_whitespace(&self, text: &String, style: &Style, ws_style: &Option<Style>) -> String {
        if let Some(ws_style) = ws_style {
            let newl = format!("{}", ws_style.paint("¶\n"));
            let space = format!("{}", ws_style.paint("•"));

            let re_nonwhitespace = Regex::new(r"[^\n ]+").unwrap();
            re_nonwhitespace.replace_all(text, |caps: &regex::Captures| {
                style.paint(&caps[0]).to_string()
            }).to_string().replace('\n', &newl).replace(' ', &space)

        } else {
            style.paint(text).to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_spaces_with_format() {
        let formatter = Formatter::default();
        let text = "hello  world";

        assert_eq!(formatter.format(text, &OutputStyle::default()), "hello world");
    }

    #[test]
    fn does_not_trim_spaces_in_monospace() {
        let formatter = Formatter::default();
        let text = "`{\n    let x = 5;\n}`";

        assert!(formatter.format(text, &OutputStyle::default()).contains("{\n    let x = 5;\n}"));
    }

    #[test]
    fn format_monospace() {
        let formatter = Formatter::default();
        let text = "To create a new variable use `let x = 5`";
        let formatted_text = formatter.format(text, &OutputStyle::default());

        assert!(!formatted_text.contains("`"));
    }

    #[test]
    fn format_monospace_adds_newline_if_there_is_none() {
        let formatter = Formatter::default();
        let text = "I have `no whitespace`";
        let formatted_text = formatter.format(text, &OutputStyle::default());

        assert!(formatted_text.contains("\n"));
    }

    #[test]
    fn format_monospace_does_not_add_additional_newlines() {
        let formatter = Formatter::default();
        let text = "I have \n\n`lots of whitespace`";
        let formatted_text = formatter.format(text, &OutputStyle::default());

        assert!(!formatted_text.contains("\n\n\n"));
    }
}

