use ansi_term::Style;
use lazy_static::lazy_static;
use regex::Regex;

use crate::outputstyle::OutputStyle;

// use lazy_static! to make sure regexes are only compiled once
lazy_static! {
    static ref RE_VARIABLE: Regex = Regex::new(r"\[\[((?s).*?)\]\]").unwrap();
    static ref RE_CONSTANT: Regex = Regex::new(r"\{\{((?s).*?)\}\}").unwrap();
    static ref RE_BOLD: Regex = Regex::new(r"<<((?s).*?)>>").unwrap();
    static ref RE_MONOSPACE: Regex = Regex::new(r"`([^`]*?)`").unwrap();
    static ref RE_MONOSPACE_OLD: Regex = Regex::new(r"```([^`]*?)```").unwrap();
    static ref RE_MONOSPACE_TRIM: Regex = Regex::new(r"\s*`\s*([^`]+?)\s*`\s*").unwrap();
    static ref RE_BACKTICK: Regex = Regex::new(r"(`[^`]+`)|([^`]+)").unwrap();
    static ref RE_SPACES: Regex = Regex::new(r" +").unwrap();
    static ref RE_NONWHITESPACE: Regex = Regex::new(r"[^\r\n ]+").unwrap();
    static ref RE_NEWLINES: Regex = Regex::new(r"\n\n\n+").unwrap();
    static ref RE_ALL_BUT_MONOSPACE: Regex =
        Regex::new(r"\[\[((?s).*?)\]\]|\{\{((?s).*?)\}\}|<<((?s).*?)>>").unwrap();
}

pub fn format_cg(text: &str, ostyle: &OutputStyle) -> String {
    if RE_MONOSPACE_OLD.is_match(&text) {
        println!(
            "{} {}\n",
            ostyle.failure.paint("WARNING"),
            "Clash contains obsolete ``` formatting, consider fixing it in the website."
        );
    }

    let mut text = format_edit_monospace(&text);
    text = format_trim_consecutive_spaces(&text);
    text = format_monospace_padding(&text);
    text = format_add_reverse_nester_tags(&text);
    text = format_paint_inner_blocks(&text, ostyle);
    format_remove_excessive_newlines(&text)
}

/// 1. Throws a warning if it finds outdated formatting
/// 2. Replaces ```text``` -> `text`
/// 3. Format whitespace around Monospace blocks.
///
/// Clashes with outdated formatting:
///     https://www.codingame.com/contribute/view/25623694f80d8f747b3fa474a33a9920335ce
///     https://www.codingame.com/contribute/view/7018d709bf39dcccec4ed9f97fb18105f64c
/// Others:
///     https://www.codingame.com/contribute/view/1222536cec20519e1a630ecc8ada367dd708b
fn format_edit_monospace(text: &str, ostyle: &OutputStyle) -> String {
    if RE_MONOSPACE_OLD.is_match(&text) {
        let msg = "Clash contains obsolete ``` formatting, consider fixing it in the website.";
        println!("{} {}\n", ostyle.failure.paint("WARNING"), msg);
    }

fn format_edit_monospace(text: &str) -> String {
    let mut result = text.replace("```", "`");

    result = RE_MONOSPACE_TRIM
        .replace_all(&result, |caps: &regex::Captures| format!("\n\n`{}`\n\n", &caps[1]))
        .to_string();

    result
}

/// If it's not inside a Monospace block, trim consecutive spaces.
fn format_trim_consecutive_spaces(text: &str) -> String {
    RE_BACKTICK
        .replace_all(&text, |caps: &regex::Captures| {
            if let Some(monospace_text) = caps.get(1) {
                monospace_text.as_str().to_string()
            } else if let Some(non_monospace_text) = caps.get(2) {
                RE_SPACES.replace_all(non_monospace_text.as_str(), " ").to_string()
            } else {
                "".to_string()
            }
        })
        .to_string()
}

/// Adds padding to Monospace blocks.

// NOTE 1: Comes before the adding of reverse nester tags so that the Monospace
//         block is not split into separate blocks and messes up the padding.
// NOTE 2: Needs to factor the fact that tags are going to be deleted later on.
fn format_monospace_padding(text: &str) -> String {
    let padded_text = RE_MONOSPACE
        .replace_all(&text, |caps: &regex::Captures| {
            let lines: Vec<&str> = caps[1].split('\n').map(|line| line.trim_end()).collect();
            let padding = lines.iter().map(|line| clean_line_size(line)).max().unwrap_or(0);
            let formatted_lines = lines
                .iter()
                .map(|&line| {
                    // Consider using .chars.count instead of .len
                    let offset = line.len() - clean_line_size(line);
                    format!("{:<width$}", line, width = padding + offset)
                })
                .collect::<Vec<String>>()
                .join("\n");
            format!("`{}`", formatted_lines)
        })
        .to_string();

    padded_text
}

/// Returns the size of a line without formatting tags. 
/// Only used for computing the padding of Monospace blocks.
fn clean_line_size(line: &str) -> usize {
    RE_ALL_BUT_MONOSPACE
        .replace_all(&line, |caps: &regex::Captures| {
            if let Some(group) = caps.get(1) {
                return group.as_str().to_string();
            }
            if let Some(group) = caps.get(2) {
                return group.as_str().to_string();
            }
            if let Some(group) = caps.get(3) {
                return group.as_str().to_string();
            }
            "".to_string()
        })
        .len()
}

/// Only supports some combinations.
/// Hacky. Based upon the fact that only 1-level nesting makes sense.
///     <<Next   [[N]]   {{3}} lines:>>
///  -> <<Next >>[[N]]<< {{3}} lines:>>
fn format_add_reverse_nester_tags(text: &str) -> String {
    // <<Next [[N]] {{3}} lines:>>
    let mut result = RE_BOLD
        .replace_all(&text, |caps: &regex::Captures| {
            let escaped_vars = RE_VARIABLE
                .replace_all(&caps[0], |inner_caps: &regex::Captures| format!(">>{}<<", &inner_caps[0]))
                .to_string();
            RE_CONSTANT
                .replace_all(&escaped_vars, |inner_caps: &regex::Captures| format!(">>{}<<", &inner_caps[0]))
                .to_string()
        })
        .to_string();

    // `Next [[N]] {{3}} <<B>> lines:`
    result = RE_MONOSPACE
        .replace_all(&result, |caps: &regex::Captures| {
            let escaped_vars = RE_VARIABLE
                .replace_all(&caps[0], |inner_caps: &regex::Captures| format!("`{}`", &inner_caps[0]))
                .to_string();
            let escaped_cons = RE_CONSTANT
                .replace_all(&escaped_vars, |inner_caps: &regex::Captures| format!("`{}`", &inner_caps[0]))
                .to_string();
            RE_BOLD
                .replace_all(&escaped_cons, |inner_caps: &regex::Captures| format!("`{}`", &inner_caps[0]))
                .to_string()
        })
        .to_string();

    // {{Next [[N]] `Mono \n and more` lines}}
    result = RE_CONSTANT
        .replace_all(&result, |caps: &regex::Captures| {
            let escaped_cons = RE_VARIABLE
                .replace_all(&caps[0], |inner_caps: &regex::Captures| {
                    format!("{}{}{}", "}}", &inner_caps[0], "{{")
                })
                .to_string();
            RE_MONOSPACE
                .replace_all(&escaped_cons, |inner_caps: &regex::Captures| {
                    format!("{}{}{}", "}}", &inner_caps[0], "{{")
                })
                .to_string()
        })
        .to_string();

    result
}

/// NOTE: [[VARIABLE]] - {{CONSTANT}} - <<BOLD>> - `MONOSPACE`
///
/// Removes formatting tags and paints the inner content accordingly:
///     [[VARIABLE]]
///  -> \u{1b}[33mVARIABLE\u{1b}[0m
///
/// For painting interactions:
///     https://www.codingame.com/contribute/view/750741cba87bb6a6ac8daf5adbe2aa083e24
///     https://www.codingame.com/contribute/view/83316b323da5dba40730dbca5c72b46ccfc9
fn format_paint_inner_blocks(text: &str, ostyle: &OutputStyle) -> String {
    let mut result = RE_VARIABLE
        .replace_all(&text, |caps: &regex::Captures| ostyle.variable.paint(&caps[1]).to_string())
        .to_string();

    result = RE_CONSTANT
        .replace_all(&result, |caps: &regex::Captures| ostyle.constant.paint(&caps[1]).to_string())
        .to_string();

    result = RE_BOLD
        .replace_all(&result, |caps: &regex::Captures| ostyle.bold.paint(&caps[1]).to_string())
        .to_string();

    result = RE_MONOSPACE
        .replace_all(&result, |caps: &regex::Captures| {
            let lines: Vec<&str> = caps[1].split('\n').collect();
            lines
                .iter()
                .map(|&line| ostyle.monospace.paint(line).to_string())
                .collect::<Vec<String>>()
                .join("\n")
        })
        .to_string();

    result
}

fn format_remove_excessive_newlines(text: &str) -> String {
    RE_NEWLINES.replace_all(&text, |_: &regex::Captures| "\n\n").trim_end().to_string()
}

/// 1. Replaces spaces with • and newlines with ⏎. Paints them with `ws_style`.
/// 2. Paints the rest with `style`.
pub fn show_whitespace(text: &str, style: &Style, ws_style: &Style) -> String {
    let newl = format!("{}\n", ws_style.paint("⏎"));
    let space = format!("{}", ws_style.paint("•"));
    let fmt_non_ws = RE_NONWHITESPACE
        .replace_all(text, |caps: &regex::Captures| style.paint(&caps[0]).to_string())
        .to_string();
    fmt_non_ws.replace('\n', &newl).replace(' ', &space)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_spaces_with_format() {
        let text = "hello  world";

        assert_eq!(format_trim_consecutive_spaces(text), "hello world");
    }

    #[test]
    fn does_not_trim_spaces_in_monospace() {
        let text = "`{\n    let x = 5;\n}`";

        assert!(format_trim_consecutive_spaces(text).contains("{\n    let x = 5;\n}"));
    }

    #[test]

    fn format_monospace_coloring_removes_backticks() {
        let text = "To create a new variable use `let x = 5`";
        let formatted_text = format_paint_inner_blocks(text, &OutputStyle::default());

        assert!(!formatted_text.contains("`"));
    }

    #[test]
    fn format_monospace_adds_newline_if_there_is_none() {
        let text = "I have `no whitespace`";
        let formatted_text = format_edit_monospace(text, &OutputStyle::default());

        assert!(formatted_text.contains("\n"));
    }

    #[test]
    fn format_monospace_trims_trailing_spaces() {
        let text = "I have `no whitespace`        and more text";
        let formatted_text = format_edit_monospace(text, &OutputStyle::default());

        assert!(!formatted_text.contains("\n "));
    }

    #[test]
    fn format_monospace_more_newlines_1() {
        let text: &str = "1text   `mono line` text";
        let formatted_text = format_edit_monospace(text, &OutputStyle::default());
        let expected = "1text\n\n`mono line`\n\ntext";

        assert_eq!(formatted_text, expected);
    }

    #[test]
    fn format_monospace_more_newlines_2() {
        let text: &str = "2text   \n\n`mono line\nnew line`  \n  text";
        let formatted_text = format_edit_monospace(text, &OutputStyle::default());
        let expected = "2text\n\n`mono line\nnew line`\n\ntext";

        assert_eq!(formatted_text, expected);
    }

    #[test]
    fn format_monospace_more_newlines_3() {
        let text: &str = "3text   \n\n   \n    `\n   \n  mono line\nnew line  \n   \n`   \n   \n   text";
        let formatted_text = format_edit_monospace(text, &OutputStyle::default());
        let expected = "3text\n\n`mono line\nnew line`\n\ntext";

        assert_eq!(formatted_text, expected);
    }

    #[test]
    fn format_monospace_more_newlines_4() {
        let text: &str = "4text\n\n`mono line`\n\ntext";
        let formatted_text = format_edit_monospace(text, &OutputStyle::default());
        let expected = "4text\n\n`mono line`\n\ntext";

        assert_eq!(formatted_text, expected);
    }

    #[test]
    fn format_correctly_add_nested_tags() {
        let text = "<<Next [[N]] {{3}} lines:>>";
        let formatted_text = format_add_reverse_nester_tags(text);
        let expected = "<<Next >>[[N]]<< >>{{3}}<< lines:>>";

        assert_eq!(formatted_text, expected);
    }

    #[test]
    fn format_correctly_paint_nested_tags() {
        let text = "<<Next [[N]] {{3}} lines:>>";
        let ostyle = &OutputStyle::default();
        let formatted_text = format_cg(text, ostyle);
        let expected = vec![
            format_cg("<<Next >>", ostyle),
            format_cg("[[N]]", ostyle),
            format_cg("<< >>", ostyle),
            format_cg("{{3}}", ostyle),
            format_cg("<< lines:>>", ostyle),
        ]
        .join("");

        assert_eq!(formatted_text, expected);
    }

    #[test]
    fn format_matches_newlines_bold() {
        let text = "<<Bold text spread \n across two lines:>>";

        assert_eq!(RE_BOLD.is_match(&text), true);
    }

    #[test]
    fn format_deals_with_newspaces() {
        let text = "Text with many\n\n\n\n\nnewlines\n\n";
        let formatted_text = format_remove_excessive_newlines(text);
        let expected = "Text with many\n\nnewlines";

        assert_eq!(formatted_text, expected);
    }
}
