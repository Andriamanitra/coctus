use ansi_term::Style;
use lazy_static::lazy_static;
use regex::Regex;

use crate::outputstyle::{merge_styles, OutputStyle};

// use lazy_static! to make sure regexes are only compiled once
lazy_static! {
    // NOTE
    // [[VARIABLE]] - {{CONSTANT}} - <<BOLD>> - `MONOSPACE`
    static ref RE_VARIABLE: Regex = Regex::new(r"\[\[((?s).*?)\]\]").unwrap();
    static ref RE_CONSTANT: Regex = Regex::new(r"\{\{((?s).*?)\}\}").unwrap();
    static ref RE_BOLD: Regex = Regex::new(r"<<((?s).*?)>>").unwrap();
    static ref RE_MONOSPACE: Regex = Regex::new(r"`([^`]*?)`").unwrap();
    static ref RE_MONOSPACE_OLD: Regex = Regex::new(r"```([^`]*?)```").unwrap();
    static ref RE_MONOSPACE_TRIM: Regex = Regex::new(r"\s*`(?: *\n)?([^`]+?)\s*`\s*").unwrap();
    static ref RE_BACKTICK: Regex = Regex::new(r"(`[^`]+`)|([^`]+)").unwrap();
    static ref RE_ALL_BUT_MONOSPACE: Regex =
        Regex::new(r"\[\[((?s).*?)\]\]|\{\{((?s).*?)\}\}|<<((?s).*?)>>").unwrap();
    static ref RE_SPACES: Regex = Regex::new(r" +").unwrap();
    static ref RE_NONWHITESPACE: Regex = Regex::new(r"[^\r\n ]+").unwrap();
    static ref RE_NEWLINES: Regex = Regex::new(r"\n\n\n+").unwrap();
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
    text = format_paint(&text, ostyle);
    format_remove_excessive_newlines(&text)
}

/// 1. Replaces \```text``` -> \`text`.
/// 2. Format whitespace around Monospace blocks.

// Clashes with outdated formatting:
//     https://www.codingame.com/contribute/view/25623694f80d8f747b3fa474a33a9920335ce
//     https://www.codingame.com/contribute/view/7018d709bf39dcccec4ed9f97fb18105f64c
// Others:
//     https://www.codingame.com/contribute/view/1222536cec20519e1a630ecc8ada367dd708b
//     https://www.codingame.com/contribute/view/6357b99de3f556ffd3edff4a4d5995c924bb
fn format_edit_monospace(text: &str) -> String {
    let mut result = text.replace("```", "`");

    result = RE_MONOSPACE_TRIM
        .replace_all(&result, |caps: &regex::Captures| format!("\n\n`{}`\n\n", &caps[1]))
        .to_string();

    result
}

/// If it's not inside a Monospace block, trim consecutive spaces.
fn format_trim_consecutive_spaces(text: &str) -> String {
    let trimmed_text = RE_BACKTICK
        .replace_all(&text, |caps: &regex::Captures| {
            if let Some(monospace_text) = caps.get(1) {
                monospace_text.as_str().to_string()
            } else if let Some(non_monospace_text) = caps.get(2) {
                RE_SPACES.replace_all(non_monospace_text.as_str(), " ").to_string()
            } else {
                "".to_string()
            }
        })
        .to_string();

    trimmed_text
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
    let amount_tag_blocks: usize = RE_ALL_BUT_MONOSPACE.find_iter(&line).count();

    line.len() - 4 * amount_tag_blocks
}

fn format_paint(text: &str, ostyle: &OutputStyle) -> String {
    // FIX: https://www.codingame.com/contribute/view/1783dda5b69105636695dc5bf51de1baf5d0

    // OTHERS
    //      https://www.codingame.com/contribute/view/750741cba87bb6a6ac8daf5adbe2aa083e24
    //      https://www.codingame.com/contribute/view/83316b323da5dba40730dbca5c72b46ccfc9

    // Hack for all tags to be two chars long
    let text = RE_MONOSPACE
        .replace_all(&text, |caps: &regex::Captures| {
            // Another hack: Quickfix for padding
            let hack_padding = &caps[1].replace('\n', "$$\n@@");
            format!("@@{}$$", hack_padding)
        })
        .to_string();

    let tag_pairs = vec![
        (ostyle.monospace, "@@", "$$"),
        (ostyle.variable, "[[", "]]"),
        (ostyle.constant, "{{", "}}"),
        (ostyle.bold, "<<", ">>"),
    ];

    let mut cur_style = Style::default();
    let mut buffer = String::new();
    let mut result = String::new();
    let mut found = false;
    let mut stack: Vec<(Style, &str)> = vec![]; // Stack of (pre_style, opening_tag)

    // Assumes no nesting <<a[[b>>c]] - should warn if it finds one ?
    for (i, c) in text.char_indices() {
        let slice = &text[i..];
        // If found is not set true by the end of the iteration, we add the char to the
        // buffer When found is true, we do nothing the next iteration since
        // tags are two chars long
        if found {
            found = false;
            continue;
        }

        for (style, tag_open, tag_close) in &tag_pairs {
            // There's a chance of a tag starting here
            if slice.starts_with(tag_open) {
                // There's definitely a tag to be parsed (grouped non-lazily)
                if slice.contains(tag_close) {
                    // NOTE (CG RULES):
                    // Tags can not nest themselves:
                    // so if the current open was already in the stack - ignore
                    // <<<<Prompt>>> => [<<Prompt]>>
                    // Although OUR double parsing looks better imo
                    // https://www.codingame.com/contribute/view/70888dd5bb12f2becdad5e6db3de8b40a77f

                    // Paint the previous buffer with the previous colour
                    // add it to the global "result" and then clear it
                    result += &cur_style.paint(&buffer).to_string();
                    buffer.clear();
                    // push cur_style to the stack to go back to it later on
                    // then update the color to paint the next buffer
                    stack.push((cur_style.clone(), tag_open));
                    cur_style = merge_styles(&cur_style, &style);
                    // Found a tag, 2 turns skip
                    found = true;
                }
            // There's a chance of a tag ending here
            } else if slice.starts_with(tag_close) {
                // Does this closing tag match the top of the stack?
                match stack.clone().last() {
                    // We did parse some opening tag before
                    Some((st, op)) => {
                        // They match, pop
                        if op == tag_open {
                            stack.pop();
                            // Paint and go back to the previous style
                            result += &cur_style.paint(&buffer).to_string();
                            buffer.clear();
                            cur_style = st.clone();
                            found = true;
                        } else {
                            // They don't match <<a]]b>> treat ]] as string
                            panic!("Nesting different tags is UNSUPPORTED")
                        }
                    }
                    // No stack means to ignore, imagine a>>b
                    _ => {}
                }
            }
        }
        if !found {
            buffer += &c.to_string();
        }
    }
    if buffer.len() > 0 {
        result += &cur_style.paint(&buffer).to_string();
        buffer.clear();
    }

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
        let formatted_text = format_paint(text, &OutputStyle::default());

        assert!(!formatted_text.contains("`"));
    }

    #[test]
    fn format_monospace_adds_newline_if_there_is_none() {
        let text = "I have `no whitespace`";
        let formatted_text = format_edit_monospace(text);

        assert!(formatted_text.contains("\n"));
    }

    #[test]
    fn format_monospace_trims_trailing_spaces() {
        let text = "I have `no whitespace`        and more text";
        let formatted_text = format_edit_monospace(text);

        assert!(!formatted_text.contains("\n "));
    }

    #[test]
    fn format_monospace_more_newlines_1() {
        let text: &str = "1text   `mono line` text";
        let formatted_text = format_edit_monospace(text);
        let expected = "1text\n\n`mono line`\n\ntext";

        assert_eq!(formatted_text, expected);
    }

    #[test]
    fn format_monospace_more_newlines_2() {
        let text: &str = "2text   \n\n`mono line\nnew line`  \n  text";
        let formatted_text = format_edit_monospace(text);
        let expected = "2text\n\n`mono line\nnew line`\n\ntext";

        assert_eq!(formatted_text, expected);
    }

    #[test]
    fn format_monospace_more_newlines_3() {
        let text: &str = "3text   \n\n   \n    `\n   \n  mono line\nnew line  \n   \n`   \n   \n   text";
        let formatted_text = format_edit_monospace(text);
        let expected = "3text\n\n`   \n  mono line\nnew line`\n\ntext";

        assert_eq!(formatted_text, expected);
    }

    #[test]
    fn format_monospace_more_newlines_4() {
        let text: &str = "4text\n\n`mono line`\n\ntext";
        let formatted_text = format_edit_monospace(text);
        let expected = "4text\n\n`mono line`\n\ntext";

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
