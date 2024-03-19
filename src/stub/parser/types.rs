use serde::Serialize;

#[derive(Clone)]
pub struct Stub {
    pub commands: Vec<Cmd>,
    pub input_comments: Vec<InputComment>,
    pub output_comment: String,
    pub statement: String,
}

// More visual than derive(Debug)
impl std::fmt::Debug for Stub {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Stub {{\n  commands: [")?;

        // Print commands recursively
        for command in &self.commands {
            write!(f, "\n    {:?}", command)?;
        }

        write!(
            f,
            "\n  ],\n  input_comments: {:?},\n  output_comment: {:?},\n  statement: {:?}\n}}",
            self.input_comments, self.output_comment, self.statement
        )
    }
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

impl Default for Stub {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct InputComment {
    pub variable: String,
    pub description: String,
}

impl InputComment {
    pub fn new(variable: String, description: String) -> Self {
        Self {
            variable,
            description,
        }
    }
}

#[derive(Serialize, Clone, Debug)]
pub enum LengthType {
    Number,
    Variable,
}

impl<'a> From<&'a str> for LengthType {
    fn from(value: &'a str) -> Self {
        match value.parse::<usize>() {
            Ok(_) => Self::Number,
            Err(_) => Self::Variable,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum VariableCommand {
    Int {
        name: String,
    },
    Float {
        name: String,
    },
    Long {
        name: String,
    },
    Bool {
        name: String,
    },
    Word {
        name: String,
        max_length: String,
        length_type: LengthType,
    },
    String {
        name: String,
        max_length: String,
        length_type: LengthType,
    },
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
