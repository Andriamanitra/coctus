mod language;

use anyhow::Result;
use itertools::Itertools;
use language::Language;
use serde_json::json;
use tera::{Context, Tera};

use self::types::VariableType;
use super::parser::{Cmd, InputComment, JoinTerm, Stub, VariableCommand};

mod types;
use types::ReadData;

pub fn render_stub(lang: &str, stub: Stub, debug_mode: bool) -> Result<String> {
    let renderer = Renderer::new(lang, stub, debug_mode)?;
    Ok(renderer.render())
}

struct Renderer {
    tera: Tera,
    lang: Language,
    stub: Stub,
    debug_mode: bool,
}

impl Renderer {
    fn new(lang_name: &str, stub: Stub, debug_mode: bool) -> Result<Renderer> {
        let lang = Language::from(lang_name);
        let tera = Tera::new(&lang.template_glob())?;
        Ok(Self {
            lang,
            tera,
            stub,
            debug_mode,
        })
    }

    fn tera_render(&self, template_name: &str, context: &mut Context) -> String {
        context.insert("debug_mode", &self.debug_mode);

        // Since these are (generally) shared across languages, it makes sense to
        // store it in the "global" context instead of accepting it as parameters.
        let format_symbols = json!({
            "Bool": "%b",
            "Float": "%f",
            "Int": "%d",
            "Long": "%l",
            "String": "%s",
            "Word": null,
        });
        context.insert("format_symbols", &format_symbols);

        self.tera
            .render(&format!("{template_name}.{}.jinja", self.lang.source_file_ext), context)
            .unwrap_or_else(|_| panic!("Failed to render {} template.", template_name))
    }

    fn render(&self) -> String {
        let mut context = Context::new();

        let statement = self.render_statement();

        let code: String = self.stub.commands.iter().map(|cmd| self.render_command(cmd)).collect();
        let code_lines: Vec<&str> = code.lines().collect();

        context.insert("statement", &statement);
        context.insert("code_lines", &code_lines);

        self.tera_render("main", &mut context)
    }

    fn render_statement(&self) -> String {
        let mut context = Context::new();
        let statement_lines: Vec<&str> = self.stub.statement.lines().collect();
        context.insert("statement_lines", &statement_lines);

        self.tera_render("statement", &mut context)
    }

    fn render_command(&self, cmd: &Cmd) -> String {
        match cmd {
            Cmd::Read(vars) => self.render_read(vars),
            Cmd::Write(message) => self.render_write(message),
            Cmd::WriteJoin(join_terms) => self.render_write_join(join_terms),
            Cmd::Loop { count, command } => self.render_loop(count, command),
            Cmd::LoopLine { count_var, variables } => self.render_loopline(count_var, variables),
        }
    }

    fn render_write(&self, message: &str) -> String {
        let mut context = Context::new();
        let messages: Vec<&str> = message.lines().map(|msg| msg.trim_end()).collect();
        context.insert("messages", &messages);

        self.tera_render("write", &mut context)
    }

    fn render_write_join(&self, terms: &Vec<JoinTerm>) -> String {
        let mut context = Context::new();
        context.insert("terms", terms);
        self.tera_render("write_join", &mut context)
    }

    fn render_read(&self, vars: &Vec<VariableCommand>) -> String {
        match vars.as_slice() {
            [var] => self.render_read_one(var),
            _ => self.render_read_many(vars),
        }
    }

    fn render_read_one(&self, var: &VariableCommand) -> String {
        let mut context = Context::new();
        let var_data = &ReadData::new(var, &self.lang);
        let comment = self.stub.input_comments.iter().find(|comment| var_data.name == comment.variable);

        context.insert("comment", &comment);
        context.insert("var", var_data);
        context.insert("type_tokens", &self.lang.type_tokens);

        self.tera_render("read_one", &mut context)
    }

    fn render_read_many(&self, vars: &[VariableCommand]) -> String {
        let mut context = Context::new();

        let read_data: Vec<ReadData> = vars
            .iter()
            .map(|var_cmd| ReadData::new(var_cmd, &self.lang))
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

        self.tera_render("read_many", &mut context)
    }

    fn render_loop(&self, count: &str, cmd: &Cmd) -> String {
        let mut context = Context::new();
        let inner_text = self.render_command(cmd);
        let count_with_case = self.lang.transform_variable_name(count);
        context.insert("count", &count_with_case);
        context.insert("inner", &inner_text.lines().collect::<Vec<&str>>());

        self.tera_render("loop", &mut context)
    }

    fn render_loopline(&self, count_var: &str, vars: &[VariableCommand]) -> String {
        let read_data: Vec<ReadData> = vars
            .iter()
            .map(|var_cmd| ReadData::new(var_cmd, &self.lang))
            .collect();
        let mut context = Context::new();
        let cased_count_var = self.lang.transform_variable_name(&String::from(count_var));
        context.insert("count_var", &cased_count_var);
        context.insert("vars", &read_data);
        context.insert("type_tokens", &self.lang.type_tokens);

        self.tera_render("loopline", &mut context)
    }
}
