use tera::{Tera, Context};

use crate::programming_language::ProgrammingLanguage;
use super::Stub;

pub fn render(lang: ProgrammingLanguage, stubs: Vec<Stub>) -> String {
    let starting_filepath = format!("config/stub_templates/{}/main.{}", lang.name, lang.source_file_ext);
    let starting_contents = std::fs::read_to_string(&starting_filepath)
        .expect("Could not find template for stub");

    // let tera = Tera::new(&starting_filepath).expect("Could not find template for stub");
    let mut context = Context::new();
    let mut code = String::new();

    for stub in stubs {
        match stub {
            Stub::Write(message) => {
                let lines: Vec<String> = message.lines().map(|line| line.to_string()).collect();
                let write_template = format!("config/stub_templates/{}/write.{}", lang.name, lang.source_file_ext);
                let content = std::fs::read_to_string(write_template).unwrap();
                let mut context = Context::new();
                context.insert("messages", &lines);
                let out = Tera::one_off(&content, &context, false).expect("Failed to render write template for stub");
                code.push_str(&out);
            },
            _ => continue,
        }
    }

    context.insert("code", &code);

    Tera::one_off(&starting_contents, &context, false).expect("Failed to render template for stub")
}

