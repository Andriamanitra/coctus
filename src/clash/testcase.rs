use serde::{Deserialize, Deserializer, Serialize};

/// `TestCase` is a deserialized representation of a test case for a Clash of
/// Code or I/O puzzle.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Testcase {
    /// `index` is the number of the test/validator, starting from 1.
    #[serde(skip_serializing, skip_deserializing)]
    pub index: usize,
    /// `title` is a human readable name for the test/validator
    #[serde(deserialize_with = "deserialize_testcase_title")]
    pub title: String,
    /// `test_in` is the input that a solution reads from STDIN
    #[serde(rename = "testIn")]
    pub test_in: String,
    /// `test_out` is the output that a solution is expected to print to STDOUT
    #[serde(rename = "testOut")]
    pub test_out: String,
    /// `is_validator` is true for test cases that are not normally visible when
    /// solving a puzzle on CodinGame.
    #[serde(rename = "isValidator")]
    pub is_validator: bool,
}

pub fn deserialize_testcases<'de, D: Deserializer<'de>>(de: D) -> Result<Vec<Testcase>, D::Error> {
    let mut testcases = Vec::<Testcase>::deserialize(de)?;

    for (i, testcase) in testcases.iter_mut().enumerate() {
        testcase.index = i + 1;
    }

    Ok(testcases)
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
            title: String,
        },
    }
    let title = match TempTitle::deserialize(de)? {
        TempTitle::Normal(title) => title,
        TempTitle::Weird { title } => title,
    };
    Ok(title)
}
