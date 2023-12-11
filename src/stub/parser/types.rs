use serde::Serialize;

#[derive(Debug, Clone)]
pub struct Stub {
    pub commands: Vec<Cmd>,
    pub input_comments: Vec<InputComment>,
    pub output_comment: String,
    pub statement: String,
}

impl Stub {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            input_comments: Vec::new(),
            output_comment: String::new(),
            statement: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct InputComment {
    pub variable: String,
    pub description: String,
}

impl InputComment {
    pub fn new(variable: String, description: String) -> Self {
        Self { variable, description }
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum VariableCommand {
    Int { name: String },
    Float { name: String },
    Long { name: String },
    Bool { name: String },
    Word { name: String, max_length: usize },
    String { name: String, max_length: usize },
}

#[derive(Serialize, Clone, Debug)]
pub enum JoinTermType {
    Literal,
    Variable,
}

#[derive(Serialize, Clone, Debug)]
pub struct JoinTerm {
    name: String,
    term_type: JoinTermType,
}

impl JoinTerm {
    pub fn new(name: String, term_type: JoinTermType) -> Self {
        Self { name, term_type }
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum Cmd {
    Read(Vec<VariableCommand>),
    Loop { count: String, command: Box<Cmd> },
    LoopLine { object: String, variables: Vec<VariableCommand> },
    Write(String),
    WriteJoin(Vec<JoinTerm>)
}
