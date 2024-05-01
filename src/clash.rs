mod public_handle;
mod test_case;

pub use public_handle::PublicHandle;
use serde::{Deserialize, Serialize};
use test_case::deserialize_testcases;
pub use test_case::TestCase;

#[derive(Debug, Serialize, Deserialize)]
pub struct Clash {
    id: u32,
    #[serde(rename = "publicHandle")]
    public_handle: PublicHandle,
    #[serde(rename = "lastVersion")]
    last_version: ClashVersion,
    #[serde(rename = "type")]
    puzzle_type: PuzzleType,
    #[serde(rename = "upVotes")]
    upvotes: i32,
    #[serde(rename = "downVotes")]
    downvotes: i32,
}

#[derive(Debug, Serialize, Deserialize)]
enum PuzzleType {
    #[serde(rename = "CLASHOFCODE")]
    Clash,
    #[serde(rename = "PUZZLE_INOUT")]
    ClassicInOut,
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
    #[serde(deserialize_with = "deserialize_testcases")]
    testcases: Vec<TestCase>,

    constraints: Option<String>,
    #[serde(rename = "stubGenerator")]
    stub_generator: Option<String>,
    #[serde(rename = "inputDescription")]
    input_description: String,
    #[serde(rename = "outputDescription")]
    output_description: String,
}

impl Clash {
    pub fn testcases(&self) -> &Vec<TestCase> {
        &self.last_version.data.testcases
    }

    pub fn codingame_link(&self) -> String {
        format!("https://www.codingame.com/contribute/view/{}", self.public_handle)
    }

    pub fn title(&self) -> &str {
        &self.last_version.data.title
    }

    pub fn statement(&self) -> &str {
        &self.last_version.data.statement
    }

    pub fn constraints(&self) -> Option<&str> {
        self.last_version.data.constraints.as_deref()
    }

    pub fn stub_generator(&self) -> Option<&str> {
        self.last_version.data.stub_generator.as_deref()
    }

    pub fn input_description(&self) -> &str {
        &self.last_version.data.input_description
    }

    pub fn output_description(&self) -> &str {
        &self.last_version.data.output_description
    }

    pub fn is_reverse(&self) -> bool {
        self.last_version.data.reverse
    }

    pub fn is_fastest(&self) -> bool {
        self.last_version.data.fastest
    }

    pub fn is_shortest(&self) -> bool {
        self.last_version.data.shortest
    }

    pub fn is_reverse_only(&self) -> bool {
        self.is_reverse() && !self.is_fastest() && !self.is_shortest()
    }
}
