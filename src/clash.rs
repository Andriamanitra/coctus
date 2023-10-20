use serde::{Serialize, Deserialize};
use regex::Regex;

#[derive(Debug, Serialize, Deserialize)]
pub struct Clash {
    id: u32,
    nickname: String,
    #[serde(rename = "publicHandle")]
    public_handle: String,
    #[serde(rename = "lastVersion")]
    last_version: ClashVersion,
    #[serde(rename = "upVotes")]
    upvotes: u32,
    #[serde(rename = "downVotes")]
    downvotes: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClashVersion {
    version: u32,
    data: ClashData,
    #[serde(rename = "statementHTML")]
    statement_html: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClashData {
    title: String,
    fastest: bool,
    reverse: bool,
    shortest: bool,
    statement: String,
    #[serde(rename = "testCases")]
    testcases: Vec<ClashTestCase>,
    constraints: String,
    #[serde(rename = "stubGenerator")]
    stub_generator: String,
    #[serde(rename = "inputDescription")]
    input_description: String,
    #[serde(rename = "outputDescription")]
    output_description: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClashTestCase {
    title: String,
    #[serde(rename = "testIn")]
    test_in: String,
    #[serde(rename = "testOut")]
    test_out: String,
    #[serde(rename = "isValidator")]
    is_validator: bool,
}

struct Formatter {
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

    fn new() -> Self {
        Formatter {
            re_variable: Regex::new(r"\[\[([^\]]+)\]\]").unwrap(),
            re_constant: Regex::new(r"\{\{([^\}]+)\}\}").unwrap(),
            re_bold: Regex::new(r"<<([^>]+)>>").unwrap(),
            // Also capture the previous '\n' if any (`Monospace` rule)
            re_monospace: Regex::new(r"\n?`([^`]+)`").unwrap(),

            fmt_variable:  "\x1b[33m".to_string(),    // Yellow
            fmt_constant:  "\x1b[34m".to_string(),    // Blue
            fmt_bold:      "\x1b[3;39m".to_string(),  // Italics
            fmt_monospace: "\x1b[39;49m".to_string(), // Do nothing for the moment
        }
    }

    fn format(&self, text: &str) -> String {
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

impl Clash {
    pub fn pretty_print(&self) {
        use std::io::Write;

        let cdata: &ClashData = &self.last_version.data;

        // TODO: --no-color flag
        let with_color = true;

        let mut buf: Vec<u8> = Vec::new();
    
        // Title and link
        writeln!(&mut buf, "\x1b[33m=== {} ===\x1b[39;49m\n", cdata.title).unwrap();
        writeln!(&mut buf, "\x1b[1;33mhttps://www.codingame.com/contribute/view/{}\x1b[0;0m\n", self.public_handle).unwrap();

        // Statement
        if with_color {
            let formatter = Formatter::new();
            let section_color = "\x1b[33m".to_string(); // Yellow
    
            writeln!(&mut buf, "{}\n", formatter.format(&cdata.statement)).unwrap();
            writeln!(&mut buf, "{}Constraints:\x1b[39;49m", section_color).unwrap();
            writeln!(&mut buf, "{}\n", formatter.format(&cdata.constraints)).unwrap();
            writeln!(&mut buf, "{}Input:\x1b[39;49m", section_color).unwrap();
            writeln!(&mut buf, "{}\n", formatter.format(&cdata.input_description)).unwrap();
            writeln!(&mut buf, "{}Output:\x1b[39;49m", section_color).unwrap();
            writeln!(&mut buf, "{}\n", formatter.format(&cdata.output_description)).unwrap();
        } else {
            writeln!(&mut buf, "{}\n", cdata.statement).unwrap();
            writeln!(&mut buf, "Constraints:").unwrap();
            writeln!(&mut buf, "{}\n", cdata.constraints).unwrap();
            writeln!(&mut buf, "Input:").unwrap();
            writeln!(&mut buf, "{}\n", cdata.input_description).unwrap();
            writeln!(&mut buf, "Output:").unwrap();
            writeln!(&mut buf, "{}\n", cdata.output_description).unwrap();
        }

        // Example testcase
        if !cdata.testcases.is_empty() {
            let example: &ClashTestCase = &cdata.testcases[0];
            let example_in: &String = &example.test_in;
            let example_out: &String = &example.test_out;
    
            writeln!(&mut buf, "\x1b[33mExample:\x1b[39;49m").unwrap();
            writeln!(&mut buf, "{}\n", &example_in).unwrap();
            writeln!(&mut buf, "\x1b[1;32m{}\x1b[39;49m", &example_out).unwrap();
        } else {
            // This should probably be a breaking error
            writeln!(&mut buf, "No testcases available.").unwrap();
        }
    
        let out_str = String::from_utf8_lossy(&buf).to_string();
        // dbg!(out_str.clone());
        println!("{}", out_str);
    }    
}
