mod language;

use itertools::Itertools;
use tera::{Tera, Context};

use language::Language;
use self::types::VariableType;

use super::parser::{Cmd, Stub, VariableCommand, InputComment, JoinTerm};

mod types;
use types::ReadData;

pub fn render_stub(lang: String, stub: Stub) -> String {
    Renderer::new(lang, stub).render()
}

struct Renderer {
    tera: Tera,
    lang: Language,
    stub: Stub,
}

impl Renderer {
    fn new(lang_name: String, stub: Stub) -> Self {
        let lang = Language::from(lang_name);
        let tera = Tera::new(&lang.template_glob())
            .expect("There are no templates for this language");
        Self { lang, tera, stub }
    }

    fn render(&self) -> String {
        let mut context = Context::new();

        let statement = self.render_statement();

        let code: String = self.stub.commands.iter()
            .map(|cmd| format!("{}\n", &self.render_command(cmd)))
            .collect::<String>().replace("\n\n", "\n");

        let code_lines: Vec<&str> = code.lines().collect();

        context.insert("statement", &statement);
        context.insert("code_lines", &code_lines);

        self.tera.render(&self.template_path("main"), &context)
            .expect("Failed to render template for stub")
    }

    fn render_statement(&self) -> String {
        let mut context = Context::new();
        let statement_lines: Vec<&str> = self.stub.statement.lines().collect();
        context.insert("statement_lines", &statement_lines);
        self.tera.render(&self.template_path("statement"), &context)
            .expect("Could not find statement template")
    }

    fn render_command(&self, cmd: &Cmd) -> String {
        match cmd {
            Cmd::Read(vars) => self.render_read(vars),
            Cmd::Write(message) => self.render_write(message),
            Cmd::WriteJoin(join_terms) => self.render_write_join(join_terms),
            Cmd::Loop { count, command } => self.render_loop(count, command),
            Cmd::LoopLine { object, variables } => self.render_loopline(object, variables),
        }
    }

    fn render_write(&self, message: &String) -> String {
        let mut context = Context::new();
        context.insert("messages", &message.lines().collect::<Vec<&str>>());
        self.tera.render(&self.template_path("write"), &context)
            .expect("Could not find write template")
    }

    fn render_write_join(&self, terms: &Vec<JoinTerm>)  -> String {
        let mut context = Context::new();
        context.insert("terms", terms);
        self.tera.render(&self.template_path("write_join"), &context)
            .expect("Could not find write template")

    }

    fn render_read(&self, vars: &Vec<VariableCommand>) -> String {
        match vars.as_slice() {
            [var] => self.render_read_one(var),
            _ => self.render_read_many(vars),
        }
    }

    fn render_read_one(&self, var: &VariableCommand) -> String {
        let mut context = Context::new();
        let var_data = &ReadData::from(var);
        let comment: Option<&InputComment> = self.stub.input_comments
            .iter().find(|comment| var_data.name == comment.variable);

        context.insert("comment", &comment);
        context.insert("var", var_data);
        context.insert("type_tokens", &self.lang.type_tokens);

        self.tera.render(&self.template_path("read_one"), &context)
            .expect("Could not find read template").trim_end().to_owned()
    }

    fn render_read_many(&self, vars: &Vec<VariableCommand>) -> String {
        let mut context = Context::new();

        let read_data: Vec<ReadData> = vars.into_iter().map(|var_cmd| ReadData::from(var_cmd)).collect();

        let comments: Vec<&InputComment> = self.stub.input_comments.iter().filter(|comment| 
            read_data.iter().any(|var_data| var_data.name == comment.variable)
        ).collect();

        let types: Vec<&VariableType> = read_data.iter()
            .map(|r| &r.var_type).unique().collect();

        match types.as_slice() {
            [single_type] => context.insert("single_type", single_type),
            _ => context.insert("single_type", &false),
        }

        context.insert("comments", &comments);
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

    fn render_loopline(&self, object: &str, vars: &Vec<VariableCommand>) -> String {
        let read_data: Vec<ReadData> = vars.into_iter().map(|var_cmd| ReadData::from(var_cmd)).collect();
        let mut context = Context::new();
        context.insert("object", &object);
        context.insert("vars", &read_data);
        context.insert("type_tokens", &self.lang.type_tokens);
        self.tera.render(&self.template_path("loopline"), &context)
            .expect("Could not find read template")
    }

    fn template_path(&self, template_name: &str) -> String {
        format!("{template_name}.{}.jinja", self.lang.source_file_ext)
    }
}

