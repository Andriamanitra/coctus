use anyhow::Result;
use serde::{Deserialize, Deserializer, Serialize};

use crate::formatter::{format_cg, show_whitespace};
use crate::outputstyle::OutputStyle;

#[derive(Debug, Serialize, Deserialize)]
pub struct Clash {
    id: u32,
    #[serde(rename = "publicHandle")]
    public_handle: String,
    #[serde(rename = "lastVersion")]
    last_version: ClashVersion,
    #[serde(rename = "type")]
    puzzle_type: String,
    #[serde(rename = "upVotes")]
    upvotes: i32,
    #[serde(rename = "downVotes")]
    downvotes: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PuzzleType {
    #[serde(rename = "CLASHOFCODE")]
    Clash,
    #[serde(rename = "PUZZLE_INOUT")]
    ClassicInOut
}

#[derive(Debug, Serialize, Deserialize)]
struct ClashVersion {
    version: u32,
    data: ClashData,
    #[serde(rename = "statementHTML")]
    statement_html: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClashData {
    title: String,

    // apparently some of these fields are missing in very old clashes, default to false
    #[serde(default)]
    fastest: bool,
    #[serde(default)]
    reverse: bool,
    #[serde(default)]
    shortest: bool,

    statement: String,
    #[serde(rename = "testCases")]
    testcases: Vec<ClashTestCase>,
    constraints: Option<String>,
    #[serde(rename = "stubGenerator")]
    stub_generator: Option<String>,
    #[serde(rename = "inputDescription")]
    input_description: String,
    #[serde(rename = "outputDescription")]
    output_description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClashTestCase {
    #[serde(deserialize_with = "deserialize_testcase_title")]
    pub title: String,
    #[serde(rename = "testIn")]
    pub test_in: String,
    #[serde(rename = "testOut")]
    pub test_out: String,
    #[serde(rename = "isValidator")]
    pub is_validator: bool,
}

// Workaround for some old clashes which have testcase title as
// { "title": { "2": "The Actual Title" } } for whatever reason
fn deserialize_testcase_title<'de, D: Deserializer<'de>>(de: D) -> Result<String, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum TempTitle {
        Normal(String),
        Weird {
            #[serde(rename = "2")]
            title: String
        },
    }
    let title = match TempTitle::deserialize(de)? {
        TempTitle::Normal(title) => title,
        TempTitle::Weird {title} => title
    };
    Ok(title)
}

impl ClashTestCase {
    pub fn styled_input(&self, ostyle: &OutputStyle) -> String {
        match ostyle.input_whitespace {
            Some(ws_style) => show_whitespace(&self.test_in, &ostyle.input, &ws_style),
            None => ostyle.input.paint(&self.test_in).to_string(),
        }
    }
    pub fn styled_output(&self, ostyle: &OutputStyle) -> String {
        match ostyle.output_whitespace {
            Some(ws_style) => show_whitespace(&self.test_out, &ostyle.output, &ws_style),
            None => ostyle.output.paint(&self.test_out).to_string(),
        }
    }
}

impl Clash {
    pub fn testcases(&self) -> &Vec<ClashTestCase> {
        &self.last_version.data.testcases
    }

    pub fn codingame_link(&self) -> String {
        format!("https://www.codingame.com/contribute/view/{}", self.public_handle)
    }

    pub fn is_reverse_only(&self) -> bool {
        let cdata: &ClashData = &self.last_version.data;
        cdata.reverse && !cdata.fastest && !cdata.shortest
    }

    pub fn print_headers(&self, ostyle: &OutputStyle) {
        let cdata: &ClashData = &self.last_version.data;
        println!("{}\n", ostyle.title.paint(format!("=== {} ===", &cdata.title)));
        println!("{}\n", ostyle.link.paint(self.codingame_link()));  
    }

    pub fn print_statement(&self, ostyle: &OutputStyle) {
        let cdata: &ClashData = &self.last_version.data;

        println!("{}\n", format_cg(&cdata.statement, ostyle));
        println!("{}\n{}\n", ostyle.title.paint("Input:"), format_cg(&cdata.input_description, ostyle));
        println!("{}\n{}\n", ostyle.title.paint("Output:"), format_cg(&cdata.output_description, ostyle));
        if let Some(constraints) = &cdata.constraints {
            println!("{}\n{}\n", ostyle.title.paint("Constraints:"), format_cg(constraints, ostyle));
        }

        let example = self.testcases().first().expect("no test cases");
        println!("{}\n{}\n{}\n{}",
            ostyle.title.paint("Example:"),
            example.styled_input(ostyle),
            ostyle.title.paint("Expected output:"),
            example.styled_output(ostyle),
        );
    }

    pub fn print_testcases(&self, ostyle: &OutputStyle, selection: Vec<usize>) {
        // Skips validators: -t 1 will print the example, -t 2 will print the second test (skipping validator 1)
        for (idx, testcase) in self.testcases().iter().filter(|t| !t.is_validator).enumerate() {
            if selection.contains(&idx) {
                let styled_title = ostyle.title.paint(format!("#{} {}", idx, testcase.title));
                println!("{}\n{}\n\n{}\n",
                    styled_title,
                    testcase.styled_input(ostyle),
                    testcase.styled_output(ostyle),
                );
            }
        }
    }
}
