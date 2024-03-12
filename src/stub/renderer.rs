mod language;

use itertools::Itertools;
use language::Language;
use tera::{Context, Tera};

use self::types::VariableType;
use super::parser::{Cmd, InputComment, JoinTerm, Stub, VariableCommand};

mod types;
use types::ReadData;

pub fn render_stub(lang: &str, stub: Stub, debug_mode: bool) -> String {
    Renderer::new(&lang, stub, debug_mode).render()
}

struct Renderer {
    tera: Tera,
    lang: Language,
    stub: Stub,
    debug_mode: bool,
}

impl Renderer {
    fn new(lang_name: &str, stub: Stub, debug_mode: bool) -> Self {
        let lang = Language::from(lang_name);
        let tera = Tera::new(&lang.template_glob()).expect("There are no templates for this language");
        Self {
            lang,
            tera,
            stub,
            debug_mode,
        }
    }

    fn tera_render(&self, template_name: &str, context: &Context) -> String {
        self.tera
            .render(&self.template_path(template_name), &context)
            .expect(&format!("Failed to render {} template.", template_name))
    }

    fn render(&self) -> String {
        let mut context = Context::new();

        let statement = self.render_statement();

        let code: String = self.stub.commands.iter().map(|cmd| self.render_command(cmd)).collect();
        let code_lines: Vec<&str> = code.lines().collect();

        context.insert("statement", &statement);
        context.insert("code_lines", &code_lines);
        context.insert("debug_mode", &self.debug_mode);

        self.tera_render("main", &context)
    }

    fn render_statement(&self) -> String {
        let mut context = Context::new();
        let statement_lines: Vec<&str> = self.stub.statement.lines().collect();
        context.insert("statement_lines", &statement_lines);
        context.insert("debug_mode", &self.debug_mode);

        self.tera_render("statement", &context)
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
        let messages: Vec<&str> = message.lines().map(|msg| msg.trim_end()).collect();
        context.insert("messages", &messages);
        context.insert("debug_mode", &self.debug_mode);

        self.tera_render("write", &context)
    }

    fn render_write_join(&self, terms: &Vec<JoinTerm>) -> String {
        let mut context = Context::new();
        context.insert("terms", terms);
        self.tera_render("write_join", &context)
    }

    fn render_read(&self, vars: &Vec<VariableCommand>) -> String {
        match vars.as_slice() {
            [var] => self.render_read_one(var),
            _ => self.render_read_many(vars),
        }
    }

    fn render_read_one(&self, var: &VariableCommand) -> String {
        let mut context = Context::new();
        let var_data = &ReadData::new(var, &self.lang.variable_format);
        let comment: Option<&InputComment> =
            self.stub.input_comments.iter().find(|comment| var_data.name == comment.variable);

        context.insert("comment", &comment);
        context.insert("var", var_data);
        context.insert("type_tokens", &self.lang.type_tokens);
        context.insert("debug_mode", &self.debug_mode);

        self.tera_render("read_one", &context)
    }

    fn render_read_many(&self, vars: &Vec<VariableCommand>) -> String {
        let mut context = Context::new();

        let read_data: Vec<ReadData> = vars
            .into_iter()
            .map(|var_cmd| ReadData::new(var_cmd, &self.lang.variable_format))
            .collect();

        let comments: Vec<&InputComment> = self
            .stub
            .input_comments
            .iter()
            .filter(|comment| read_data.iter().any(|var_data| var_data.name == comment.variable))
            .collect();

        let types: Vec<&VariableType> = read_data.iter().map(|r| &r.var_type).unique().collect();

        match types.as_slice() {
            [single_type] => context.insert("single_type", single_type),
            _ => context.insert("single_type", &false),
        }

        context.insert("comments", &comments);
        context.insert("vars", &read_data);
        context.insert("type_tokens", &self.lang.type_tokens);
        context.insert("debug_mode", &self.debug_mode);

        self.tera_render("read_many", &context)
    }

    fn render_loop(&self, count: &String, cmd: &Box<Cmd>) -> String {
        let mut context = Context::new();
        let inner_text = self.render_command(&cmd);
        let count_with_case = self.lang.variable_format.convert(count);
        context.insert("count", &count_with_case);
        context.insert("inner", &inner_text.lines().collect::<Vec<&str>>());
        context.insert("debug_mode", &self.debug_mode);

        self.tera_render("loop", &context)
    }

    fn render_loopline(&self, object: &str, vars: &Vec<VariableCommand>) -> String {
        let read_data: Vec<ReadData> = vars
            .into_iter()
            .map(|var_cmd| ReadData::new(var_cmd, &self.lang.variable_format))
            .collect();
        let mut context = Context::new();
        let object_with_case = self.lang.variable_format.convert(&String::from(object));
        context.insert("object", &object_with_case);
        context.insert("vars", &read_data);
        context.insert("type_tokens", &self.lang.type_tokens);
        context.insert("debug_mode", &self.debug_mode);

        self.tera_render("loopline", &context)
    }

    fn template_path(&self, template_name: &str) -> String {
        format!("{template_name}.{}.jinja", self.lang.source_file_ext)
    }
}
