use tera::{Tera, Context};

use crate::programming_language::ProgrammingLanguage;
use super::Stub;

pub struct Renderer {
    tera: Tera,
    lang: ProgrammingLanguage,
}

impl Renderer {
    pub fn new(lang: ProgrammingLanguage) -> Self {
        let tera = Tera::new(&lang.template_glob())
            .expect("There are no templates for this language");
        Self { lang, tera }
    }

    pub fn render(&self, stubs: Vec<Stub>) -> String {
        let mut context = Context::new();
        let mut code = String::new();

        let code_lines: Vec<String> = stubs.iter().map(|stub| {
            match stub {
                Stub::Write(message) => self.render_write(message),
                _ => String::from(""),
            }
        }).collect();

        context.insert("code_lines", &code_lines);

        self.tera.render(&format!("main.{}", self.lang.source_file_ext), &context)
            .expect("Failed to render template for stub")
    }

    fn render_write(&self, message: &String) -> String {
        let mut context = Context::new();
        context.insert("messages", &message.lines().collect::<Vec<&str>>());
        self.tera.render(&self.template_path("write"), &context)
            .expect("Could not find write template")
    }

    fn template_path(&self, template_name: &str) -> String {
        format!("{template_name}.{}", self.lang.source_file_ext)
    }
}




