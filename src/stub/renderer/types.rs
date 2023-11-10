use serde::Serialize;

use crate::stub::parser::types::VariableCommand;

#[derive(Debug, Clone, Serialize)]
pub enum VariableType {
    Int,
    Float,
    Long,
    Bool,
    Word,
    String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReadData {
    pub name: String,
    pub var_type: VariableType,
    pub max_length: Option<usize>,
}

impl From<&VariableCommand> for ReadData {
    fn from(value: &VariableCommand) -> Self {
        let (name, var_type, max_length) = match value {
            VariableCommand::Int { name } => (name, VariableType::Int, None),
            VariableCommand::Float { name } => (name, VariableType::Float, None),
            VariableCommand::Long { name } => (name, VariableType::Long, None),
            VariableCommand::Bool { name } => (name, VariableType::Bool, None),
            VariableCommand::Word { name, max_length } => (name, VariableType::Word, Some(*max_length)),
            VariableCommand::String { name, max_length } => (name, VariableType::String, Some(*max_length)),
        };

        Self { name: name.clone(), var_type, max_length }
    }
}
