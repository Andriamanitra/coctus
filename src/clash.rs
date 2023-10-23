use serde::{Serialize, Deserialize, Deserializer};
use regex::Regex;

mod formatter;
use formatter::Formatter;

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
            if let Some(constraints) = &cdata.constraints {
                writeln!(&mut buf, "{}Constraints:\x1b[39;49m", section_color).unwrap();
                writeln!(&mut buf, "{}\n", formatter.format(&constraints)).unwrap();
            }
            writeln!(&mut buf, "{}Input:\x1b[39;49m", section_color).unwrap();
            writeln!(&mut buf, "{}\n", formatter.format(&cdata.input_description)).unwrap();
            writeln!(&mut buf, "{}Output:\x1b[39;49m", section_color).unwrap();
            writeln!(&mut buf, "{}\n", formatter.format(&cdata.output_description)).unwrap();
        } else {
            writeln!(&mut buf, "{}\n", cdata.statement).unwrap();
            if let Some(constraints) = &cdata.constraints {
                writeln!(&mut buf, "Constraints:").unwrap();
                writeln!(&mut buf, "{}\n", constraints).unwrap();
            }
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

            // For visibility: turn spaces into dim "•" and newlines into dim "¶"
            let visual_example_in = Regex::new(r"\n")
              .unwrap()
              .replace_all(&example_in, "\x1b[2m¶\n\x1b[0m");
            let visual_example_in = Regex::new(r" ")
              .unwrap()
              .replace_all(&visual_example_in, "\x1b[2m•\x1b[0m");

            writeln!(&mut buf, "\x1b[33mExample:\x1b[39;49m").unwrap();
            writeln!(&mut buf, "{}\n", &visual_example_in).unwrap();
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
