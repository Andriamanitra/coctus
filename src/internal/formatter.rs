use ansi_term::Style;
use lazy_static::lazy_static;
use regex::Regex;

use super::outputstyle::OutputStyle;

// use lazy_static! to make sure regexes are only compiled once
lazy_static! {
    static ref RE_MONOSPACE: Regex = Regex::new(r"`([^`]*?)`").unwrap();
    static ref RE_MONOSPACE_OLD: Regex = Regex::new(r"```([^`]*?)```").unwrap();
    static ref RE_MONOSPACE_TRIM: Regex = Regex::new(r"\s*`(?: *\n)?([^`]+?)\n?`\s*").unwrap();
    static ref RE_BACKTICK: Regex = Regex::new(r"(`[^`]+`)|([^`]+)").unwrap();
    static ref RE_ALL_BUT_MONOSPACE: Regex =
        Regex::new(r"\[\[((?s).*?)\]\]|\{\{((?s).*?)\}\}|<<((?s).*?)>>").unwrap();
    static ref RE_SPACES: Regex = Regex::new(r" +").unwrap();
    static ref RE_NONWHITESPACE: Regex = Regex::new(r"[^\r\n ]+").unwrap();
    static ref RE_NEWLINES: Regex = Regex::new(r"\n\n\n+").unwrap();
}

/// Formats `text` that contains CodinGame formatting into a string
/// styled with ANSI terminal escape sequences. The supported formatting
/// directives are:
/// ```text
/// [[VARIABLE]] - {{CONSTANT}} - <<BOLD>> - `MONOSPACE`
/// ```
pub fn format_cg(text: &str, ostyle: &OutputStyle) -> String {
    if RE_MONOSPACE_OLD.is_match(text) {
        eprintln!(
            "{} Clash contains obsolete ``` formatting, consider fixing it in the website.\n",
            ostyle.failure.paint("WARNING"),
        );
    }

    let mut text = format_edit_monospace(text);
    text = format_trim_consecutive_spaces(&text);
    text = format_monospace_padding(&text);
    text = format_paint(&text, ostyle);
    format_remove_excessive_newlines(&text)
}

/// Replaces triple quoted monospace blocks with single quoted ones
/// (\`\`\`text\`\`\` -> \`text\`) and adds newlines around them. As of 2023
/// CodinGame does not actually support triple quoted monospace but clash
/// authors occasionally use them anyway in which case CodinGame renders
/// something like
/// ```html
/// <pre></pre>
/// <pre>text</pre>
/// <pre></pre>
/// ```
fn format_edit_monospace(text: &str) -> String {
    let mut result = text.replace("```", "`");

    result = RE_MONOSPACE_TRIM
        .replace_all(&result, |caps: &regex::Captures| format!("\n\n`{}`\n\n", &caps[1]))
        .to_string();

    result
}

/// Replaces multiple consecutive spaces with just one space. Consecutive spaces
/// inside monospace blocks are left as-is.
fn format_trim_consecutive_spaces(text: &str) -> String {
    RE_BACKTICK
        .replace_all(text, |caps: &regex::Captures| {
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

/// Pads lines in multiline monospace blocks with spaces to make them the same
/// length. Attempts to factor in that formatting tags are going to be deleted.
fn format_monospace_padding(text: &str) -> String {
    RE_MONOSPACE
        .replace_all(text, |caps: &regex::Captures| {
            let lines: Vec<&str> = caps[1].split('\n').collect();
            let padding = lines.iter().map(|line| clean_line_size(line)).max().unwrap_or(0);
            let formatted_lines = lines
                .iter()
                .map(|&line| {
                    // Consider using .chars.count instead of .len
                    let offset = line.len() - clean_line_size(line);
                    format!("`{:<width$}`", line, width = padding + offset)
                })
                .collect::<Vec<String>>()
                .join("\n");
            formatted_lines
        })
        .to_string()
}

/// Calculate the length of a string (in bytes) without CodinGame's formatting
/// tags.
fn clean_line_size(line: &str) -> usize {
    let amount_tag_blocks: usize = RE_ALL_BUT_MONOSPACE.find_iter(line).count();

    line.len() - 4 * amount_tag_blocks
}

fn paint_parts<'a>(text: &'a str, style_tag_pairs: &[(Style, &str, &str)]) -> Vec<ansi_term::ANSIString<'a>> {
    let mut parts = Vec::<ansi_term::ANSIString<'a>>::new();

    let mut cur_style = Style::default();
    let mut buffer = String::new();
    let mut skip_until = 0;
    let mut num_warnings = 0;
    let mut stack: Vec<(Style, &str)> = vec![]; // Stack of (pre_style, opening_tag)

    for (i, c) in text.char_indices() {
        // Skip formatting tags by not adding them to the buffer.
        if i < skip_until {
            continue;
        }

        let slice = &text[i..];
        for (style, tag_open, tag_close) in style_tag_pairs {
            if slice.starts_with(tag_close) {
                // Does this opening tag match the top of the stack?
                if let Some((style, opening)) = stack.to_owned().last() {
                    if opening == tag_open {
                        stack.pop();
                        // Paint and go back to the previous style
                        parts.push(cur_style.paint(buffer.to_string()));
                        buffer.clear();
                        cur_style = *style;

                        // Found a valid tag, skip it
                        skip_until = i + tag_close.len();
                        break
                    } else {
                        // Closing tag doesn't match the opening tag: ignore it and treat it as a normal
                        // character
                        // For example: `a\n>>b` (ok), or <<a[[b>>c]] (invalid).

                        if num_warnings == 0 {
                            eprintln!(
                                "{} Bad formatting: tried to close {:?} with {:?}",
                                Style::new().on(ansi_term::Color::Red).paint("WARNING"),
                                opening,
                                tag_close,
                            );
                        }
                        num_warnings += 1;
                    }
                }
            }

            if slice.starts_with(tag_open) {
                // There's definitely a tag to be parsed (grouped non-lazily)
                if slice.contains(tag_close) {
                    // NOTE (CG RULES):
                    // Tags can not nest themselves:
                    //     <<<<Prompt>>> => [<<Prompt]>>
                    // So if the current open was already in the stack: ignore.

                    // Paint the previous buffer with the previous colour
                    // add it to the global "result" and then clear it
                    parts.push(cur_style.paint(buffer.to_owned()));
                    buffer.clear();
                    // push cur_style to the stack to go back to it later on
                    // then update the color to paint the next buffer
                    stack.push((cur_style, tag_open));
                    cur_style = nested_style(style, &cur_style);

                    // Found a valid tag, skip it
                    skip_until = i + tag_open.len();
                } else {
                    // Opening tag that is never closed: ignore it and treat it as a normal
                    // character
                    if num_warnings == 0 {
                        eprintln!(
                            "{} Bad formatting: ignoring {:?} that is never closed",
                            Style::new().on(ansi_term::Color::Red).paint("WARNING"),
                            tag_open
                        );
                    }
                    num_warnings += 1;
                }
                break
            }
        }
        if i >= skip_until {
            buffer.push(c);
        }
    }

    for (_, tag_open) in stack {
        // Opening tag was never closed
        if num_warnings == 0 {
            eprintln!(
                "{} Bad formatting: {:?} was never closed",
                Style::new().on(ansi_term::Color::Red).paint("WARNING"),
                tag_open
            );
        }
        num_warnings += 1;
    }

    if !buffer.is_empty() {
        parts.push(cur_style.paint(buffer.to_string()));
    }

    parts
}

fn format_paint(text: &str, ostyle: &OutputStyle) -> String {
    let tag_pairs = vec![
        (ostyle.monospace, "`", "`"),
        (ostyle.variable, "[[", "]]"),
        (ostyle.constant, "{{", "}}"),
        (ostyle.bold, "<<", ">>"),
    ];

    let parts = paint_parts(text, &tag_pairs);
    ansi_term::ANSIStrings(&parts).to_string()
}

fn format_remove_excessive_newlines(text: &str) -> String {
    RE_NEWLINES.replace_all(text, |_: &regex::Captures| "\n\n").trim_end().to_string()
}

/// 1. Replaces spaces with • and newlines with ⏎. Paints them with `ws_style`.
/// 2. Paints the rest with `style`.
pub fn show_whitespace(text: &str, style: &Style, ws_style: &Option<Style>) -> String {
    match ws_style {
        None => style.paint(text).to_string(),
        Some(ws_style) => {
            let newl = format!("{}\n", ws_style.paint("⏎"));
            let space = format!("{}", ws_style.paint("•"));
            let fmt_non_ws = RE_NONWHITESPACE
                .replace_all(text, |caps: &regex::Captures| style.paint(&caps[0]).to_string())
                .to_string();
            fmt_non_ws.replace('\n', &newl).replace(' ', &space)
        }
    }
}

/// Construct a new style that is the combination of `inner` and `outer` style.
/// The new style keeps all attributes from `inner` and adds ones from `outer`
/// if the corresponding attribute in `inner` is the default for that attribute.
fn nested_style(inner: &Style, outer: &Style) -> Style {
    Style {
        foreground: inner.foreground.or(outer.foreground),
        background: inner.background.or(outer.background),
        is_bold: inner.is_bold || outer.is_bold,
        is_italic: inner.is_italic || outer.is_italic,
        is_underline: inner.is_underline || outer.is_underline,
        is_blink: inner.is_blink || outer.is_blink,
        is_dimmed: inner.is_dimmed || outer.is_dimmed,
        is_reverse: inner.is_reverse || outer.is_reverse,
        is_hidden: inner.is_hidden || outer.is_hidden,
        is_strikethrough: inner.is_strikethrough || outer.is_strikethrough,
    }
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

        assert!(!formatted_text.contains('`'));
    }

    #[test]
    fn format_monospace_adds_newline_if_there_is_none() {
        let text = "I have `no whitespace`";
        let formatted_text = format_edit_monospace(text);

        assert!(formatted_text.contains('\n'));
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
        let expected = "3text\n\n`   \n  mono line\nnew line  \n   `\n\ntext";

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
    fn format_deals_with_newspaces() {
        let text = "Text with many\n\n\n\n\nnewlines\n\n";
        let formatted_text = format_remove_excessive_newlines(text);
        let expected = "Text with many\n\nnewlines";

        assert_eq!(formatted_text, expected);
    }

    #[test]
    fn painting_simple() {
        use ansi_term::Color::*;

        let red = Style::default().fg(Red);
        let green = Style::default().fg(Green);
        let blue = Style::default().fg(Blue);

        let tag_pairs = vec![
            (Style::default(), "{{", "}}"),
            (blue, "[[", "]]"),
            (red, "<<", ">>"),
            (green, "`", "`"),
        ];

        let parts = paint_parts("vv<<RED>>ww`GREEN`xx[[BLUE]]yy{{DEFAULT}}zz", &tag_pairs);
        println!("\n{}", ansi_term::ANSIStrings(&parts));
        assert_eq!(parts[0], ansi_term::ANSIString::from("vv"));
        assert_eq!(parts[1], red.paint("RED"));
        assert_eq!(parts[2], ansi_term::ANSIString::from("ww"));
        assert_eq!(parts[3], green.paint("GREEN"));
        assert_eq!(parts[4], ansi_term::ANSIString::from("xx"));
        assert_eq!(parts[5], blue.paint("BLUE"));
        assert_eq!(parts[6], ansi_term::ANSIString::from("yy"));
        assert_eq!(parts[7], ansi_term::ANSIString::from("DEFAULT"));
        assert_eq!(parts[8], ansi_term::ANSIString::from("zz"));
        assert_eq!(parts.len(), 9);
    }

    #[test]
    fn painting_nested() {
        use ansi_term::Color::{Blue, Red};
        let inner_style = Style::default().fg(Red);
        let outer_style = Style::default().on(Blue);

        let tag_pairs = vec![(outer_style, "`", "`"), (inner_style, "<<", ">>")];

        let parts = paint_parts("AA`BB<<CC>>DD`EE", &tag_pairs);
        println!("\n{}", ansi_term::ANSIStrings(&parts));
        assert_eq!(parts[0], ansi_term::ANSIString::from("AA"));
        assert_eq!(parts[1], outer_style.paint("BB"));
        assert_eq!(parts[2], inner_style.on(Blue).paint("CC"));
        assert_eq!(parts[3], outer_style.paint("DD"));
        assert_eq!(parts[4], ansi_term::ANSIString::from("EE"));
        assert_eq!(parts.len(), 5);
    }

    #[test]
    /// Test formatting that really shouldn't exist – it doesn't really matter
    /// what the output is (since the formatting is not well-defined anyway)
    /// as long as we don't crash
    fn painting_weird_and_invalid() {
        let ostyle = OutputStyle::default();
        println!("\nInvalid formatting tests:");
        let examples = [
            "<<AA[[BB>>CC]]",
            "```",
            "XX```YY",
            "<<]]",
            "[[[[AA]]",
            "[[[[AA]]]]",
            "<<[[AA>>]]>>",
        ];

        for (idx, original) in examples.iter().enumerate() {
            let formatted = format_paint(original, &ostyle);
            println!(" {}. {:?} becomes \"{}\"", idx + 1, original, formatted);
        }
    }
}
