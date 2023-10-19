use serde::{Serialize, Deserialize};

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

impl Clash {
    pub fn pretty_print(&self) {
        let cdata = &self.last_version.data;
        println!("\x1b[34;47m=== {} ===\x1b[39;49m\n", cdata.title);
        println!("https://www.codingame.com/contribute/view/{}\n", self.public_handle);
        println!("{}\n", cdata.statement);
        println!("Constraints:\n{}\n", cdata.constraints);
        println!("Input:\n{}\n", cdata.input_description);
        println!("Output:\n{}\n", cdata.output_description);
    }
}
