use tera::{Tera, Context};

use crate::programming_language::ProgrammingLanguage;
use super::parser::{Cmd, Stub, VariableCommand};

mod types;
use types::ReadData;

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

        // Transform self.stub.commands into successive strings
        let commands: Vec<String> = self.stub.commands.iter().map(|cmd| {
            let cmd_str = self.render_command(cmd);
            dbg!(&cmd_str);
            format!("{}\n", cmd_str.as_str())
        }).collect();

        context.insert("commands", &commands);

        let result = self.tera.render(&format!("main.{}.jinja", self.lang.source_file_ext), &context)
            .expect("Failed to render template for stub");

        result
    }

    fn render_command(&self, cmd: &Cmd) -> String {
        match cmd {
            Cmd::Read(vars) => self.render_read(vars),
            Cmd::Write(message) => self.render_write(message),
            Cmd::Loop { count, command } => self.render_loop(count, command),
            _ => String::from(""),
        }
    }

    fn render_write(&self, message: &String) -> String {
        let mut context = Context::new();
        context.insert("messages", &message.lines().collect::<Vec<&str>>());
        self.tera.render(&self.template_path("write"), &context)
            .expect("Could not find write template")
    }

    fn render_read(&self, vars: &Vec<VariableCommand>) -> String {
        let read_data: Vec<ReadData> = vars.into_iter().map(|var_cmd| ReadData::from(var_cmd)).collect();
        let mut context = Context::new();
        context.insert("vars", &read_data);
        self.tera.render(&self.template_path("read"), &context)
            .expect("Could not find read template").trim_end().to_owned()
    }

    fn render_loop(&self, count: &String, cmd: &Box<Cmd>) -> String {
        let mut context = Context::new();
        let rendered_cmd: Vec<String> = self.render_command(&cmd).lines().map(|s|s.to_owned()).collect();
        // let rendered_cmd = self.render_command(&cmd);
        context.insert("count", &count);
        context.insert("inner", &rendered_cmd);
        self.tera.render(&self.template_path("loop"), &context)
            .expect("Could not find loop template").trim_end().to_owned()
    }

    fn template_path(&self, template_name: &str) -> String {
        format!("{template_name}.{}.jinja", self.lang.source_file_ext)
    }
}




