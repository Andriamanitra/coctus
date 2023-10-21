use serde::{Serialize, Deserialize, Deserializer};

mod formatter;
use formatter::Formatter;
mod app;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Clash {
    id: u32,
    public_handle: String,
    last_version: ClashVersion,
    #[serde(rename = "upVotes")]
    upvotes: i32,
    #[serde(rename = "downVotes")]
    downvotes: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClashVersion {
    version: u32,
    data: ClashData,
    statement_html: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    stub_generator: Option<String>,
    input_description: String,
    output_description: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClashTestCase {
    #[serde(deserialize_with = "deserialize_testcase_title")]
    title: String,
    test_in: String,
    test_out: String,
    is_validator: bool,

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

#[cfg(test)]
mod tests {
    use super::*;
    use app::App;
    use directories::ProjectDirs;

    fn app() -> App {
        let project_dirs = ProjectDirs::from("com", "Clash CLI", "clash")
            .expect("Unable to find project directory");
        App::new(project_dirs.data_dir())
    }

    fn decimal_fraction_percentage_clash() -> String {
        let clash_file = app().clash_dir.join("58951a69f66d23586be8084cb0969c637b07.json");
        std::fs::read_to_string(clash_file)
            .expect("Cannot find test for clash. Please run status command.")
    }

    #[test]
    fn do_not_panic_when_missing_statement_html() {
        serde_json::from_str::<Clash>(&decimal_fraction_percentage_clash()).unwrap();
    }
}