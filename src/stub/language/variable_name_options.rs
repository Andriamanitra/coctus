use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;

use crate::stub::VariableCommand;

lazy_static! {
    static ref SC_WORD_BREAK: Regex = Regex::new(r"([a-z])([A-Z])").unwrap();
    static ref PC_WORD_BREAK: Regex = Regex::new(r"([A-Z]*)([A-Z][a-z])").unwrap();
    static ref PC_WORD_END: Regex = Regex::new(r"([A-Z])([A-Z]*$)").unwrap();
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)]
pub enum Casing {
    SnakeCase,
    CamelCase,
    PascalCase,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct VariableNameOptions {
    pub casing: Casing,
    pub allow_uppercase_vars: Option<bool>,
    pub keywords: Vec<String>,
}

fn is_uppercase_string(string: &str) -> bool {
    string.chars().all(|c| c.is_uppercase())
}

impl VariableNameOptions {
    pub fn transform_variable_name(&self, variable_name: &str) -> String {
        // CG has special treatment for variables with all uppercase identifiers.
        // In most languages they remain uppercase regardless of variable format.
        // In others (such as ruby where constants are uppercase) they get downcased.
        let converted_variable_name = match (is_uppercase_string(variable_name), self.allow_uppercase_vars) {
            (true, Some(false)) => variable_name.to_lowercase(),
            (true, _) => variable_name.to_string(),
            (false, _) => self.convert(variable_name),
        };

        self.escape_keywords(converted_variable_name)
    }

    pub fn transform_variable_command(&self, var: &VariableCommand) -> VariableCommand {
        VariableCommand {
            ident: self.transform_variable_name(&var.ident),
            var_type: var.var_type.clone(),
            input_comment: var.input_comment.clone(),
            max_length: var.max_length.clone(),
        }
    }

    pub fn escape_keywords(&self, variable_name: String) -> String {
        if self.keywords.contains(&variable_name) {
            format!("_{variable_name}")
        } else {
            variable_name
        }
    }

    fn convert(&self, variable_name: &str) -> String {
        match self.casing {
            Casing::SnakeCase => Self::convert_to_snake_case(variable_name),
            Casing::PascalCase => Self::covert_to_pascal_case(variable_name),
            Casing::CamelCase => Self::covert_to_camel_case(variable_name),
        }
    }

    fn convert_to_snake_case(variable_name: &str) -> String {
        SC_WORD_BREAK
            .replace_all(variable_name, |caps: &regex::Captures| {
                format!("{}_{}", &caps[1], &caps[2].to_lowercase())
            })
            .to_lowercase()
            .to_string()
    }

    fn covert_to_pascal_case(variable_name: &str) -> String {
        variable_name[0..1].to_uppercase() + &Self::pascalize(&variable_name[1..])
    }

    fn covert_to_camel_case(variable_name: &str) -> String {
        variable_name[0..1].to_lowercase() + &Self::pascalize(&variable_name[1..])
    }

    fn pascalize(variable_slice: &str) -> String {
        let start_replaced = PC_WORD_BREAK.replace_all(variable_slice, |caps: &regex::Captures| {
            format!("{}{}", &caps[1].to_lowercase(), &caps[2])
        });

        PC_WORD_END
            .replace_all(&start_replaced, |caps: &regex::Captures| {
                format!("{}{}", &caps[1], &caps[2].to_lowercase())
            })
            .to_string()
    }
}
