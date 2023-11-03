use tera::{Tera, Context};

use crate::programming_language::ProgrammingLanguage;
use super::{Command, Stub};

pub fn render_stub(lang: ProgrammingLanguage, stub: Stub) -> String {
    let rend = Renderer::new(lang, stub);
    rend.render()
}

struct Renderer {
    tera: Tera,
    lang: ProgrammingLanguage,
    stub: Stub,
}

impl Renderer {
    fn new(lang: ProgrammingLanguage, stub: Stub) -> Self {
        let tera = Tera::new(&lang.template_glob())
            .expect("There are no templates for this language");
        Self { lang, tera, stub }
    }

    fn render(&self) -> String {
        let mut context = Context::new();

        let commands: Vec<String> = self.stub.commands.iter().map(|stub| {
            match stub {
                Command::Write(message) => self.render_write(message),
                _ => String::from(""),
            }
        }).collect();

        context.insert("commands", &commands);

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




