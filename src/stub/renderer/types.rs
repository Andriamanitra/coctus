use serde::Serialize;

use crate::stub::parser::types::VariableCommand;

#[derive(Debug, Clone, Serialize, Hash, PartialEq)]
pub enum VariableType {
    Int,
    Float,
    Long,
    Bool,
    Word,
    String,
}

impl Eq for VariableType {
    fn assert_receiver_is_total_eq(&self) {}
}

#[derive(Debug, Clone, Serialize)]
pub struct ReadData {
    pub name: String,
    pub var_type: VariableType,
    pub type_token_key: String,
    pub max_length: Option<usize>,
}

impl From<&VariableCommand> for ReadData {
    fn from(value: &VariableCommand) -> Self {
        let (name, var_type, key, max_length) = match value {
            VariableCommand::Int { name } => (name, VariableType::Int, "int", None),
            VariableCommand::Float { name } => (name, VariableType::Float, "float", None),
            VariableCommand::Long { name } => (name, VariableType::Long, "long", None),
            VariableCommand::Bool { name } => (name, VariableType::Bool, "bool", None),
            VariableCommand::Word { name, max_length } => (name, VariableType::Word, "word", Some(*max_length)),
            VariableCommand::String { name, max_length } => (name, VariableType::String, "string", Some(*max_length)),
        };

        Self { name: name.clone(), var_type, max_length, type_token_key: key.to_owned() }
    }
}
