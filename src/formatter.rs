use crate::outputstyle::OutputStyle;
use ansi_term::Style;
use lazy_static::lazy_static;
use regex::Regex;

// use lazy_static! to make sure regexes are only compiled once
lazy_static! {
    static ref RE_VARIABLE: Regex       = Regex::new(r"\[\[((?s).*?)\]\]").unwrap();
    static ref RE_CONSTANT: Regex       = Regex::new(r"\{\{((?s).*?)\}\}").unwrap();
    static ref RE_BOLD: Regex           = Regex::new(r"<<((?s).*?)>>").unwrap();
    static ref RE_MONOSPACE: Regex      = Regex::new(r"`([^`]*?)`").unwrap();
    static ref RE_MONOSPACE_TRIM: Regex = Regex::new(r"\n? *(`[^`]*`) *").unwrap();
    static ref RE_BACKTICK: Regex       = Regex::new(r"(`[^`]+`)|([^`]+)").unwrap();
    static ref RE_SPACES: Regex         = Regex::new(r" +").unwrap();
    static ref RE_NONWHITESPACE: Regex  = Regex::new(r"[^\r\n ]+").unwrap();
}

/// Format Codingame statement that contains special formatting syntax
/// [[VARIABLE]] - {{CONSTANT}} - <<BOLD>> - `MONOSPACE`
pub fn format_cg(text: &str, ostyle: &OutputStyle) -> String {
    // Trim consecutive spaces (imitates html behaviour)
    // But only if it's not in a Monospace block (between backticks ``)
    let mut result = RE_BACKTICK.replace_all(text, |caps: &regex::Captures| {
        if let Some(backtick_text) = caps.get(1) {
            backtick_text.as_str().to_string()
        } else if let Some(non_backtick_text) = caps.get(2) {
            RE_SPACES.replace_all(non_backtick_text.as_str(), " ").to_string()
        } else {
            "".to_string()
        }
    }).to_string();

    // Make sure monospace blocks are on their own line, and remove extra
    // whitespace around them
    result = RE_MONOSPACE_TRIM.replace_all(&result, |caps: &regex::Captures| {
        format!("\n{}\n", &caps[1])
    }).to_string();

    // Nested tags (only some combinations).
    // Hacky - it's based upon the fact that only 1-level nesting makes sense.
    // Adds reverse nester brackets so that the following replacement logic will work.
    // i.e : <<Next [[N]] {{3}} lines:>> becomes <<Next >>[[N]]<< {{3}} lines:>>

    // <<Next [[N]] {{3}} lines:>>
    result = RE_BOLD.replace_all(&result, |caps: &regex::Captures| {
        let escaped_vars = RE_VARIABLE.replace_all(&caps[0], |inner_caps: &regex::Captures| {
            format!(">>{}<<", &inner_caps[0])
        }).to_string();
        RE_CONSTANT.replace_all(&escaped_vars, |inner_caps: &regex::Captures| {
            format!(">>{}<<", &inner_caps[0])
        }).to_string()
    }).to_string();

    // `Next [[N]] {{3}} lines:`
    result = RE_MONOSPACE.replace_all(&result, |caps: &regex::Captures| {
        let escaped_vars = RE_VARIABLE.replace_all(&caps[0], |inner_caps: &regex::Captures| {
            format!("`{}`", &inner_caps[0])
        }).to_string();
        RE_CONSTANT.replace_all(&escaped_vars, |inner_caps: &regex::Captures| {
            format!("`{}`", &inner_caps[0])
        }).to_string()
    }).to_string();

    // {{Next [[N]] `Mono \n and more` lines}}
    result = RE_CONSTANT.replace_all(&result, |caps: &regex::Captures| {
        let escaped_cons = RE_VARIABLE.replace_all(&caps[0], |inner_caps: &regex::Captures| {
            format!("{}{}{}", "}}", &inner_caps[0], "{{")
        }).to_string();
        RE_MONOSPACE.replace_all(&escaped_cons, |inner_caps: &regex::Captures| {
            format!("{}{}{}", "}}", &inner_caps[0], "{{")
        }).to_string()
    }).to_string();

    result = RE_VARIABLE.replace_all(&result, |caps: &regex::Captures| {
        ostyle.variable.paint(&caps[1]).to_string()
    }).to_string();

    result = RE_CONSTANT.replace_all(&result, |caps: &regex::Captures| {
        ostyle.constant.paint(&caps[1]).to_string()
    }).to_string();
    
    result = RE_BOLD.replace_all(&result, |caps: &regex::Captures| {
        ostyle.bold.paint(&caps[1]).to_string()
    }).to_string();

    result = RE_MONOSPACE.replace_all(&result, |caps: &regex::Captures| {
        ostyle.monospace.paint(&caps[1]).to_string()
    }).to_string();

    result
}

/// Replaces spaces with "•" and newlines with "¶" and paints them with
/// `ws_style`. Other characters are painted with `style`.
pub fn show_whitespace(text: &str, style: &Style, ws_style: &Style) -> String {
    let newl  = format!("{}\n", ws_style.paint("⏎"));
    let space = format!("{}", ws_style.paint("•"));
    let fmt_non_ws = RE_NONWHITESPACE.replace_all(text, |caps: &regex::Captures| {
        style.paint(&caps[0]).to_string()
    }).to_string();
    fmt_non_ws.replace('\n', &newl).replace(' ', &space)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_spaces_with_format() {
        let text = "hello  world";

        assert_eq!(format_cg(text, &OutputStyle::default()), "hello world");
    }

    #[test]
    fn does_not_trim_spaces_in_monospace() {
        let text = "`{\n    let x = 5;\n}`";

        assert!(format_cg(text, &OutputStyle::default()).contains("{\n    let x = 5;\n}"));
    }

    #[test]
    fn format_monospace() {
        let text = "To create a new variable use `let x = 5`";
        let formatted_text = format_cg(text, &OutputStyle::default());

        assert!(!formatted_text.contains("`"));
    }

    #[test]
    fn format_monospace_adds_newline_if_there_is_none() {
        let text = "I have `no whitespace`";
        let formatted_text = format_cg(text, &OutputStyle::default());

        assert!(formatted_text.contains("\n"));
    }

    #[test]
    fn format_monospace_trims_trailing_spaces() {
        let text = "I have `no whitespace`        and more text";
        let formatted_text = format_cg(text, &OutputStyle::default());

        assert!(!formatted_text.contains("\n "));
    }

    #[test]
    fn format_monospace_does_not_add_additional_newlines() {
        let text = "I have \n\n`lots of whitespace`";
        let formatted_text = format_cg(text, &OutputStyle::default());

        assert!(!formatted_text.contains("\n\n\n"));
    }

    #[test]
    fn format_nested() {
        let text = "<<Next [[N]] {{3}} lines:>>";
        let ostyle = &OutputStyle::default();
        let formatted_text = format_cg(text, ostyle);
        let expected = vec![
            format_cg("<<Next >>", ostyle),
            format_cg("[[N]]", ostyle),
            format_cg("<< >>", ostyle),
            format_cg("{{3}}", ostyle),
            format_cg("<< lines:>>", ostyle)
        ].join("");

        assert_eq!(formatted_text, expected);
    }

    #[test]
    fn matches_newlines_bold() {
        let text = "<<Bold text spread \n across two lines:>>";
        
        assert_eq!(RE_BOLD.is_match(&text), true);
    }
}

