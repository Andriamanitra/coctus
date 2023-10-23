use serde::{Serialize, Deserialize, Deserializer};
use ansi_term::Style;
use ansi_term::Colour;

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
        let with_color = false;
        let formatter = Formatter::new(with_color);

        let mut buf: Vec<u8> = Vec::new();
    
        // Title and link
        let intro_colour = if with_color {
            Style::new().fg(Colour::Yellow)
        } else {
            Style::default()
        };
        writeln!(
            &mut buf, "{}\n",
            intro_colour.paint(format!("=== {} ===", &cdata.title))
        ).unwrap();
        writeln!(
            &mut buf, "{}\n",
            intro_colour.bold().paint(format!(
                "https://www.codingame.com/contribute/view/{}", self.public_handle)
            )
        ).unwrap();

        // Statement
        let titles_colour = if with_color {
            Style::new().fg(Colour::Yellow)
        } else {
            Style::default()
        };
        writeln!(&mut buf, "{}\n", formatter.format(&cdata.statement)).unwrap();
        writeln!(&mut buf, "{}", titles_colour.paint("Input:")).unwrap();
        writeln!(&mut buf, "{}\n", formatter.format(&cdata.input_description)).unwrap();
        writeln!(&mut buf, "{}", titles_colour.paint("Output:")).unwrap();
        writeln!(&mut buf, "{}\n", formatter.format(&cdata.output_description)).unwrap();
        if let Some(constraints) = &cdata.constraints {
            writeln!(&mut buf, "{}", titles_colour.paint("Constraints:")).unwrap();
            writeln!(&mut buf, "{}\n", formatter.format(&constraints)).unwrap();
        }

        // Example testcase
        let example: &ClashTestCase = &cdata.testcases[0];
        writeln!(&mut buf, "{}", titles_colour.paint("Example:")).unwrap();
        writeln!(&mut buf, "{}\n", &formatter.add_visibility(
            &example.test_in, Style::default())
        ).unwrap();
        writeln!(&mut buf, "{}", &formatter.add_visibility(
            &example.test_out, Style::new().bold().fg(Colour::Green))
        ).unwrap();
    
        let out_str = String::from_utf8_lossy(&buf).to_string();
        // dbg!(out_str.clone());
        println!("{}", out_str);
    }    
}
