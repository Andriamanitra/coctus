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
pub struct Var {
    pub name: String,
    pub t: T,
    pub max_length: usize,
}

#[derive(Debug, Clone)]
pub struct InputComment {
    variable: String,
    description: String,
}

impl InputComment {
    pub fn new(variable: String, description: String) -> Self {
        Self { variable, description }
    }
}

#[derive(Debug, Clone)]
pub enum VariableCommand {
  Int { name: String },
  Float { name: String },
  Long { name: String },
  Bool { name: String },
  Word { name: String, max_length: usize },
  String { name: String, max_length: usize },
}

impl Var {
    pub fn new(name: String, t: T) -> Var {
        Var { name, t, max_length: 0 }
    }

    pub fn new_length(name: String, t: T, max_length: usize) -> Var {
        Var { name, t, max_length}
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum T {
    Int,
    Float,
    Long,
    Bool,
    Word,
    String,
}

#[derive(Debug, Clone, Serialize)]
pub enum Cmd {
  Read(Vec<Var>),
  Loop { count: String, command: Box<Cmd> },
  LoopLine { object: String, variables: Vec<Var> },
  Write(String),
}
