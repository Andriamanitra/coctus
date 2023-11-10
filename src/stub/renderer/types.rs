use serde::Serialize;

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

impl ReadData {
    pub fn new_length(name: String, t: VariableType, max_length: Option<usize>) -> Self {
        Self { name, var_type: t, max_length}
    }
}
