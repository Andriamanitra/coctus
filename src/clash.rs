use anyhow::Result;
use serde::{Deserialize, Deserializer, Serialize};

use crate::formatter::Formatter;

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

#[derive(Debug, Serialize, Deserialize)]
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

impl Clash {
    pub fn testcases(&self) -> &Vec<ClashTestCase> {
        &self.last_version.data.testcases
    }

    pub fn pretty_print(&self, style: OutputStyle) -> Result<()> {
        let cdata: &ClashData = &self.last_version.data;
        let formatter = Formatter::default();

        // Title and link
        println!("{}\n", style.title.paint(format!("=== {} ===", &cdata.title)));
        println!("{}\n", style.link.paint(format!("https://www.codingame.com/contribute/view/{}", self.public_handle)));

        // Statement
        println!("{}\n", formatter.format(&cdata.statement, &style));
        println!("{}\n{}\n", style.title.paint("Input:"), formatter.format(&cdata.input_description, &style));
        println!("{}\n{}\n", style.title.paint("Output:"), formatter.format(&cdata.output_description, &style));
        if let Some(constraints) = &cdata.constraints {
            println!("{}\n{}\n", style.title.paint("Constraints:"), formatter.format(constraints, &style));
        }

        // Example testcase
        let example: &ClashTestCase = &cdata.testcases[0];
        let header = style.title.paint("Example:").to_string();
        println!("{}", formatter.format_testcase(example, &style, header));

        Ok(())
    }

    pub fn print_testcases(&self, style: OutputStyle) -> Result<()> {
        let formatter = Formatter::default();
        let mut test_count: u8 = 0;
        for testcase in self.testcases() {
            if testcase.is_validator { continue; }
            test_count += 1;
            let header = format!("(TEST {}) {}", test_count, &testcase.title);
            let test_in = formatter.format_testcase(testcase, &style, header);
            println!("{}\n", test_in);
        }

        Ok(())
    }
}
