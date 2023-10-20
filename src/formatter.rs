use regex::Regex;

pub struct Formatter {
    re_variable: Regex,
    re_constant: Regex,
    re_bold: Regex,
    re_monospace: Regex,

    fmt_variable: String, 
    fmt_constant: String, 
    fmt_bold: String, 
    fmt_monospace: String, 
}

impl Formatter {
    // TODO: finish support `Monospace` (Newline trimming)
    // For testing `Monospace`: 23214afcdb23616e230097d138bd872ea7c75
    // TODO: support nested formatting <<Next [[n]] lines:>>

    pub fn new() -> Self {
        Formatter {
            re_variable: Regex::new(r"\[\[(.+?)\]\]").unwrap(),
            re_constant: Regex::new(r"\{\{(.+?)\}\}").unwrap(),
            re_bold: Regex::new(r"<<(.+?)>>").unwrap(),
            // Also capture the previous '\n' if any (`Monospace` rule)
            re_monospace: Regex::new(r"\n?`([^`]+)`").unwrap(),

            fmt_variable:  "\x1b[33m".to_string(),    // Yellow
            fmt_constant:  "\x1b[34m".to_string(),    // Blue
            fmt_bold:      "\x1b[3;39m".to_string(),  // Italics
            fmt_monospace: "\x1b[39;49m".to_string(), // Do nothing for the moment
        }
    }

    pub fn format(&self, text: &str) -> String {
        // Trim consecutive spaces (imitates html behaviour)
        // But only if it's not in a Monospace block (between backticks ``)
        let re_backtick = Regex::new(r"`([^`]+)`|([^`]+)").unwrap();
        let re_spaces = Regex::new(r" +").unwrap();

        let _trimmed_spaces = re_backtick.replace_all(text, |caps: &regex::Captures| {
            if let Some(backtick_text) = caps.get(1) {
                backtick_text.as_str().to_string()
            } else if let Some(non_backtick_text) = caps.get(2) {
                re_spaces.replace_all(non_backtick_text.as_str(), " ").to_string()
            } else {
                "".to_string()
            }
        }).as_bytes().to_vec();
        let trimmed_spaces = std::str::from_utf8(&_trimmed_spaces).unwrap();

        let formatted_var = self.re_variable.replace_all(trimmed_spaces, |caps: &regex::Captures| {
            format!("{}{}\x1b[39;49m", &self.fmt_variable, &caps[1])
        });
        let formatted_con = self.re_constant.replace_all(&formatted_var, |caps: &regex::Captures| {
            format!("{}{}\x1b[39;49m", &self.fmt_constant, &caps[1])
        });
        let formatted_bold = self.re_bold.replace_all(&formatted_con, |caps: &regex::Captures| {
            format!("{}{}\x1b[0;0m", &self.fmt_bold, &caps[1])
        });

        let formatted_mono = self.re_monospace.replace_all(&formatted_bold, |caps: &regex::Captures| {
            // Extra newline at the start for monospace
            format!("\n{}{}\x1b[39;49m", &self.fmt_monospace, &caps[1])
        });
        return formatted_mono.to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_spaces_with_format() {
        let formatter = Formatter::new();
        let text = "hello  world";

        assert_eq!(formatter.format(text), "hello world");
    }

    #[test]
    fn does_not_trim_spaces_in_monospace() {
        let formatter = Formatter::new();
        let text = "`{\n    let x = 5;\n}`";

        assert!(formatter.format(text).contains("{\n    let x = 5;\n}"));
    }

    #[test]
    fn format_bold_text() {
        let formatter = Formatter::new();
        let text = "Regular <<very bold>> text";
        let formatted_text = formatter.format(text);

        assert!(!formatted_text.contains("<<"));
        assert!(formatted_text.contains(&formatter.fmt_bold));
    }

    #[test]
    fn format_tricky_bold_text() {
        let formatter = Formatter::new();
        let text = "In life <<the trick is to realize that 2 > 3>>";
        let formatted_text = formatter.format(text);

        assert!(!formatted_text.contains("<<"));
        assert!(formatted_text.contains(&formatter.fmt_bold));
    }

    #[test]
    fn format_constants() {
        let formatter = Formatter::new();
        let text = "Some values {{never ever}} change";
        let formatted_text = formatter.format(text);

        assert!(!formatted_text.contains("{{"));
        assert!(formatted_text.contains(&formatter.fmt_constant));
    }

    #[test]
    fn format_tricky_constants() {
        let formatter = Formatter::new();
        let text = "When Santa smiles it looks like {{ :} }}";
        let formatted_text = formatter.format(text);

        assert!(!formatted_text.contains("{{"));
        assert!(formatted_text.contains(&formatter.fmt_constant));
    }

    #[test]
    fn format_variables() {
        let formatter = Formatter::new();
        let text = "The correct value of [[x]] is something you won't find";
        let formatted_text = formatter.format(text);

        assert!(!formatted_text.contains("[["));
        assert!(formatted_text.contains(&formatter.fmt_variable));
    }

    #[test]
    fn format_tricky_variables() {
        let formatter = Formatter::new();
        let text = "Vector item [[v[1]]] is nil if len is 1";
        let formatted_text = formatter.format(text);

        assert!(!formatted_text.contains("[["));
        assert!(formatted_text.contains(&formatter.fmt_variable));
    }

    #[test]
    fn format_monospace() {
        let formatter = Formatter::new();
        let text = "To create a new variable use `let x = 5`";
        let formatted_text = formatter.format(text);

        assert!(!formatted_text.contains("`"));
    }

    #[test]
    fn format_monospace_adds_newline_if_there_is_none() {
        let formatter = Formatter::new();
        let text = "I have `no whitespace`";
        let formatted_text = formatter.format(text);

        assert!(formatted_text.contains("\n"));
    }

    #[test]
    fn format_monospace_does_not_add_additional_newlines() {
        let formatter = Formatter::new();
        let text = "I have \n\n`lots of whitespace`";
        let formatted_text = formatter.format(text);

        assert!(!formatted_text.contains("\n\n\n"));
    }

    #[test]
    fn nest_variable_and_constant_in_bold() {
        let formatter = Formatter::new();
        let text = "Some things <<are {{bold}}, some others are [[extra]]>>";
        let formatted_text = formatter.format(text);

        assert!(!formatted_text.contains("<<"));
        assert!(formatted_text.contains(&formatter.fmt_bold));
        assert!(!formatted_text.contains("{{"));
        assert!(formatted_text.contains(&formatter.fmt_constant));
        assert!(!formatted_text.contains("[["));
        assert!(formatted_text.contains(&formatter.fmt_variable));
    }

    #[test]
    fn nest_constant_and_variable_in_monospace() {
        let formatter = Formatter::new();
        let text = "`[[status]] = {{EXCELLENT}}.freeze`";
        let formatted_text = formatter.format(text);

        assert!(!formatted_text.contains("{{"));
        assert!(formatted_text.contains(&formatter.fmt_constant));
        assert!(!formatted_text.contains("[["));
        assert!(formatted_text.contains(&formatter.fmt_variable));

        assert!(!formatted_text.contains("`"));
    }
}

