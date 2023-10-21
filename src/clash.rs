use serde::{Serialize, Deserialize};
mod app;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Clash {
    id: u32,
    nickname: String,
    public_handle: String,
    last_version: ClashVersion,
    #[serde(rename = "upVotes")]
    upvotes: u32,
    #[serde(rename = "downVotes")]
    downvotes: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClashVersion {
    version: u32,
    data: ClashData,
    #[serde(default)] // statementHtml may not always exist
    statement_html: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClashData {
    title: String,
    fastest: bool,
    reverse: bool,
    shortest: bool,
    statement: String,
    #[serde(rename = "testCases")]
    testcases: Vec<ClashTestCase>,
    constraints: String,
    stub_generator: String,
    input_description: String,
    output_description: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClashTestCase {
    title: String,
    test_in: String,
    test_out: String,
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
