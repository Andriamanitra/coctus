use itertools::Itertools;
use tera::{Tera, Context};

use crate::programming_language::ProgrammingLanguage;
use super::parser::{Cmd, Stub, VariableCommand, InputComment};

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

        let statement_str = self.render_statement();
        // Transform self.stub.commands into successive strings
        let commands: Vec<String> = self.stub.commands.iter().map(|cmd| {
            let cmd_str = self.render_command(cmd);
            // TODO: Make this less stupid
            format!("{}\n", cmd_str.as_str()).replace("\n\n", "\n").replace("\n\n", "\n")
        }).collect();

        context.insert("statement", &statement_str);
        context.insert("commands", &commands);

        self.tera.render(&format!("main.{}.jinja", self.lang.source_file_ext), &context)
            .expect("Failed to render template for stub")
    }

    fn render_statement(&self) -> String {
        let statement_lines: Vec<&str> = self.stub.statement.lines().collect();
        let mut context = Context::new();
        context.insert("statement_lines", &statement_lines);
        self.tera.render(&self.template_path("statement"), &context)
            .expect("Could not find statement template")
    }

    fn render_command(&self, cmd: &Cmd) -> String {
        match cmd {
            Cmd::Read(vars) => self.render_read(vars),
            Cmd::Write(message) => self.render_write(message),
            Cmd::Loop { count, command } => self.render_loop(count, command),
            Cmd::LoopLine { object, variables } => self.render_loopline(object, variables),
            Cmd::WriteJoin(join_terms) => String::from(""),
        }
    }

    fn render_write(&self, message: &String) -> String {
        let mut context = Context::new();
        context.insert("messages", &message.lines().collect::<Vec<&str>>());
        self.tera.render(&self.template_path("write"), &context)
            .expect("Could not find write template")
    }

    fn render_read(&self, vars: &Vec<VariableCommand>) -> String {
        match vars.len() {
            1 => self.render_read_one(vars.first().unwrap()),
            _ => self.render_read_many(vars),
        }
    }

    fn render_read_one(&self, var: &VariableCommand) -> String {
        let mut context = Context::new();
        let comment: Option<&InputComment> = self.stub.input_comments
            .iter().find(|comment| var.name() == &comment.variable);

        context.insert("comment", &comment);
        context.insert("var", &ReadData::from(var));
        context.insert("type_tokens", &self.lang.type_tokens);

        self.tera.render(&self.template_path("read_one"), &context)
            .expect("Could not find read template").trim_end().to_owned()
    }

    fn render_read_many(&self, vars: &Vec<VariableCommand>) -> String {
        let mut context = Context::new();

        let read_data: Vec<ReadData> = vars.into_iter().map(|var_cmd| ReadData::from(var_cmd)).collect();

        let relevant_comments: Vec<&InputComment> = self.stub.input_comments.iter().filter(|comment| 
            vars.iter().any(|var_cmd| var_cmd.name() == &comment.variable)
        ).collect();

        let single_type: bool  = read_data.iter().unique_by(|r| &r.var_type).count() == 1;

        if single_type {
            context.insert("single_type", &read_data.first().unwrap().type_token_key);
        }

        context.insert("comments", &relevant_comments);
        context.insert("vars", &read_data);
        context.insert("type_tokens", &self.lang.type_tokens);

        self.tera.render(&self.template_path("read_many"), &context)
            .expect("Could not find read template").trim_end().to_owned()
    }

    fn render_loop(&self, count: &String, cmd: &Box<Cmd>) -> String {
        let mut context = Context::new();
        let inner_text = self.render_command(&cmd);
        context.insert("count", &count);
        context.insert("inner", &inner_text.lines().collect::<Vec<&str>>());
        self.tera.render(&self.template_path("loop"), &context)
            .expect("Could not find loop template")
    }

    fn template_path(&self, template_name: &str) -> String {
        format!("{template_name}.{}.jinja", self.lang.source_file_ext)
    }

    fn render_loopline(&self, object: &str, vars: &Vec<VariableCommand>) -> String {
        let read_data: Vec<ReadData> = vars.into_iter().map(|var_cmd| ReadData::from(var_cmd)).collect();
        let mut context = Context::new();
        context.insert("object", &object);
        context.insert("vars", &read_data);
        context.insert("type_tokens", &self.lang.type_tokens);
        self.tera.render(&self.template_path("loopline"), &context)
            .expect("Could not find read template")
    }
}




