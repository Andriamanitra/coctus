use crate::{outputstyle::OutputStyle, clash::ClashTestCase};
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
            re_variable:  Regex::new(r"\[\[(.*?)\]\]").unwrap(),
            re_constant:  Regex::new(r"\{\{(.*?)\}\}").unwrap(),
            re_bold:      Regex::new(r"<<(.*?)>>").unwrap(),
            re_monospace: Regex::new(r"`([^`]*?)`").unwrap(),
        }
    }
}

impl Formatter {
    // For testing `Monospace`: 23214afcdb23616e230097d138bd872ea7c75

    pub fn format(&self, text: &str, ostyle: &OutputStyle) -> String {
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

        // Deal with newlines in Monospace (irrespective of colour styles)
        let re_monospace_trim = Regex::new(r"\n? *(`[^`]*`) *").unwrap();
        result = re_monospace_trim.replace_all(&result, |caps: &regex::Captures| {
            format!("\n{}\n", &caps[1])
        }).to_string();

        // Nested tags (only some combinations).
        // Hacky - it's based upon the fact that only 1-level nesting makes sense.
        // Adds reverse nester brackets so that the following replacement logic will work.
        //      i.e : <<Next [[N]] {{3}} lines:>> becomes <<Next >>[[N]]<< {{3}} lines:>>
        // <<Next [[N]] {{3}} lines:>>
        result = self.re_bold.replace_all(&result, |caps: &regex::Captures| {
            let escaped_vars = self.re_variable.replace_all(&caps[0], |inner_caps: &regex::Captures| {
                format!(">>{}<<", &inner_caps[0])
            }).to_string();
            self.re_constant.replace_all(&escaped_vars, |inner_caps: &regex::Captures| {
                format!(">>{}<<", &inner_caps[0])
            }).to_string()
        }).to_string();
        // `Next [[N]] {{3}} lines:`
        result = self.re_monospace.replace_all(&result, |caps: &regex::Captures| {
            let escaped_vars = self.re_variable.replace_all(&caps[0], |inner_caps: &regex::Captures| {
                format!("`{}`", &inner_caps[0])
            }).to_string();
            self.re_constant.replace_all(&escaped_vars, |inner_caps: &regex::Captures| {
                format!("`{}`", &inner_caps[0])
            }).to_string()
        }).to_string();
        // {{Next [[N]] lines}}
        result = self.re_constant.replace_all(&result, |caps: &regex::Captures| {
            self.re_variable.replace_all(&caps[0], |inner_caps: &regex::Captures| {
                format!("{}{}{}", "}}", &inner_caps[0], "{{")
            }).to_string()
        }).to_string();

        // Replace tags with corresponding styles
        let regex_style_pairs = vec![
            (&self.re_variable, &ostyle.variable),
            (&self.re_constant, &ostyle.constant),
            (&self.re_bold, &ostyle.bold),
            (&self.re_monospace, &ostyle.monospace),
        ];
        
        for (regex, style) in regex_style_pairs {
            if let Some(style) = style {
                result = regex.replace_all(&result, |caps: &regex::Captures| {
                    style.paint(&caps[1]).to_string()
                }).to_string();
            }
        }
        
        result
    }

    pub fn format_testcase_as_example(&self, testcase: &ClashTestCase, ostyle: &OutputStyle, header: &str) -> String {
        let header = ostyle.title.paint(header);
        let test_in = self.show_whitespace(
            &testcase.test_in, &ostyle.input_example, &ostyle.input_whitespace);
        let test_out = self.show_whitespace(
            &testcase.test_out, &ostyle.output_example, &ostyle.output_whitespace);
        
        format!("{}\n{}\n\n{}", &header, &test_in, &test_out)
    }

    // For visibility: turn spaces into "•" and newlines into "¶"
    pub fn show_whitespace(&self, text: &str, style: &Style, ws_style: &Option<Style>) -> String {
        // ws_style for the whitespace, style for the non whitespace text.
        if let Some(ws_style) = ws_style {
            let newl  = format!("{}", ws_style.paint("¶\n"));
            let space = format!("{}", ws_style.paint("•"));
            let re_nonwhitespace = Regex::new(r"[^\r\n ]+").unwrap();
            re_nonwhitespace.replace_all(text, |caps: &regex::Captures| {
                style.paint(&caps[0]).to_string()
            }).to_string().replace('\n', &newl).replace(' ', &space)
        } else {
            // If ws_style is None, we apply style to everything.
            // NOTE: we may consider just applying style to the non whitespace text to be consistent
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
    fn format_monospace_trims_trailing_spaces() {
        let formatter = Formatter::default();
        let text = "I have `no whitespace`        and more text";
        let formatted_text = formatter.format(text, &OutputStyle::default());

        assert!(!formatted_text.contains("\n "));
    }

    #[test]
    fn format_monospace_does_not_add_additional_newlines() {
        let formatter = Formatter::default();
        let text = "I have \n\n`lots of whitespace`";
        let formatted_text = formatter.format(text, &OutputStyle::default());

        assert!(!formatted_text.contains("\n\n\n"));
    }

    #[test]
    fn format_nested() {
        let formatter = Formatter::default();
        let text = "<<Next [[N]] lines:>>";
        let formatted_text = formatter.format(text, &OutputStyle::default());
        let part1 = formatter.format("<<Next >>", &OutputStyle::default());
        let part2 = formatter.format("[[N]]", &OutputStyle::default());
        let part3 = formatter.format("<< lines:>>", &OutputStyle::default());
        let final_text = format!("{}{}{}", part1, part2, part3);

        assert_eq!(formatted_text, final_text);
    }    
}

