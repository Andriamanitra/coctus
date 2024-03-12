use serde::Serialize;

use super::language::VariableNameFormat;
use crate::stub::parser::types::VariableCommand;
use crate::stub::parser::LengthType;

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
    pub max_length: Option<String>,
    pub length_type: Option<LengthType>,
}

impl ReadData {
    // VariableNameFormat is just the case (snake_case, pascal_case etc.)
    pub fn new(value: &VariableCommand, name_format: &VariableNameFormat) -> Self {
        use {VariableCommand as VC, VariableType as VT};

        let (name, var_type, max_length, length_type) = match value {
            VC::Int { name } => (name, VT::Int, None, None),
            VC::Float { name } => (name, VT::Float, None, None),
            VC::Long { name } => (name, VT::Long, None, None),
            VC::Bool { name } => (name, VT::Bool, None, None),
            VC::Word {
                name,
                max_length,
                length_type,
            }
            | VC::String {
                name,
                max_length,
                length_type,
            } => {
                let length = match length_type {
                    LengthType::Variable => name_format.convert(max_length),
                    LengthType::Number => max_length.clone(),
                };

                let var_type = if let VC::Word { .. } = value {
                    VT::Word
                } else {
                    VT::String
                };

                (name, var_type, Some(length), Some(length_type.clone()))
            }
        };

        Self {
            name: name_format.convert(name),
            var_type,
            max_length,
            length_type,
        }
    }
}
