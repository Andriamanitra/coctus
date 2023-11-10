use tera::{Tera, Context};

use crate::programming_language::ProgrammingLanguage;
use super::parser::{Cmd, Stub, Var, T};

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
        // let mut context = Context::new();
        let _tmp: T = T::Int;

        // Transform self.stub.commands into successive strings
        let commands: Vec<String> = self.stub.commands.iter().map(|cmd| {
            let cmd_str = self.render_command(cmd);
            dbg!(&cmd_str);
            format!("{}\n", cmd_str.as_str().trim_end())
        }).collect();

        // context.insert("commands", &commands);

        // let result = self.tera.render(&format!("main.{}", self.lang.source_file_ext), &context)
        //     .expect("Failed to render template for stub");

        commands.join("")
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

    fn render_read(&self, cmds: &Vec<Var>) -> String {
        let mut context = Context::new();
        context.insert("messages", &cmds);
        self.tera.render(&self.template_path("read"), &context)
            .expect("Could not find read template")
    }

    fn render_loop(&self, count: &String, cmd: &Box<Cmd>) -> String {
        let mut context = Context::new();
        let rendered_cmd = self.render_command(&cmd);
        context.insert("count", &count);
        context.insert("inner", &rendered_cmd);
        self.tera.render(&self.template_path("loop"), &context)
            .expect("Could not find loop template")
    }

    fn template_path(&self, template_name: &str) -> String {
        format!("{template_name}.{}", self.lang.source_file_ext)
    }
}




