use serde::Serialize;

use super::language::VariableNameFormat;
use crate::stub::parser::types::VariableCommand;

#[derive(Debug, Clone, Serialize, Hash, PartialEq, Eq)]
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

impl ReadData {
    pub fn new(value: &VariableCommand, name_format: &VariableNameFormat) -> Self {
        let (name, var_type, max_length) = match value {
            VariableCommand::Int { name } => (name, VariableType::Int, None),
            VariableCommand::Float { name } => (name, VariableType::Float, None),
            VariableCommand::Long { name } => (name, VariableType::Long, None),
            VariableCommand::Bool { name } => (name, VariableType::Bool, None),
            VariableCommand::Word { name, max_length } => (name, VariableType::Word, Some(*max_length)),
            VariableCommand::String { name, max_length } => (name, VariableType::String, Some(*max_length)),
        };

        Self {
            name: name_format.convert(name),
            var_type,
            max_length,
        }
    }
}
