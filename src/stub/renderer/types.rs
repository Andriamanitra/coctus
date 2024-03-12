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
    // VariableNameFormat is just the case (snake_case, pascal_case etc.)
    pub fn new(value: &VariableCommand, name_format: &VariableNameFormat) -> Self {
        use {VariableCommand as VC, VariableType as VT};

        let (name, var_type, max_length) = match value {
            VC::Int { name } => (name, VT::Int, None),
            VC::Float { name } => (name, VT::Float, None),
            VC::Long { name } => (name, VT::Long, None),
            VC::Bool { name } => (name, VT::Bool, None),
            VC::Word { name, max_length } => (name, VT::Word, Some(*max_length)),
            VC::String { name, max_length } => (name, VT::String, Some(*max_length)),
        };

        Self {
            name: name_format.convert(name),
            var_type,
            max_length,
        }
    }
}
