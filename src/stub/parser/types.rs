use serde::Serialize;

#[derive(Clone, Default)]
pub struct Stub {
    pub commands: Vec<Cmd>,
    pub statement: String,
}

// More visual than derive(Debug)
impl std::fmt::Debug for Stub {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Stub {{\n  commands: [")?;
        for command in &self.commands {
            write!(f, "\n    {:?}", command)?;
        }
        write!(f, "\n  ],\n  statement: {:?}\n}}", self.statement)
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
pub enum VarType {
    Int,
    Float,
    Long,
    Bool,
    Word,
    String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariableCommand {
    pub ident: String,
    pub var_type: VarType,
    pub max_length: Option<String>,
    pub input_comment: String,
}

impl VariableCommand {
    pub fn new(ident: String, var_type: VarType, max_length: Option<String>) -> VariableCommand {
        VariableCommand {
            ident,
            var_type,
            max_length,
            input_comment: String::new(),
        }
    }
}

#[derive(Serialize, Clone, Debug)]
pub enum JoinTermType {
    Literal,
    Variable,
}

#[derive(Serialize, Clone, Debug)]
pub struct JoinTerm {
    pub name: String,
    pub term_type: JoinTermType,
}

impl JoinTerm {
    pub fn new(name: String, term_type: JoinTermType) -> Self {
        Self { name, term_type }
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum Cmd {
    Read(Vec<VariableCommand>),
    Loop {
        count_var: String,
        command: Box<Cmd>,
    },
    LoopLine {
        count_var: String,
        variables: Vec<VariableCommand>,
    },
    Write {
        text: String,
        output_comment: String,
    },
    WriteJoin {
        join_terms: Vec<JoinTerm>,
        output_comment: String,
    },
}
