use itertools::Itertools;
use serde::Deserialize;

use crate::stub::VariableCommand;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)]
enum Casing {
    SnakeCase,
    KebabCase,
    CamelCase,
    PascalCase,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct VariableNameOptions {
    casing: Casing,
    #[serde(default)]
    allow_uppercase_vars: bool,
    keywords: Vec<String>,
    /// Set to true if the casing of keywords does not matter in the language (default=false)
    /// Used when determining if an identifier needs to be renamed when generating a stub.
    #[serde(default)]
    case_insensitive_keywords: bool,
}

fn is_uppercase_string(string: &str) -> bool {
    string.chars().all(|c| c.is_uppercase())
}

impl VariableNameOptions {
    pub(in crate::stub) fn transform_variable_name(&self, variable_name: &str) -> String {
        // CG has special treatment for variables with all uppercase identifiers.
        // In most languages they remain uppercase regardless of variable format.
        // In others (such as ruby where constants are uppercase) they get downcased.
        let converted_variable_name = match is_uppercase_string(variable_name) {
            true if self.allow_uppercase_vars => variable_name.to_string(),
            true => variable_name.to_lowercase(),
            false => self.convert(variable_name),
        };

        self.escape_keywords(converted_variable_name)
    }

    pub(in crate::stub) fn transform_variable_command(&self, var: &VariableCommand) -> VariableCommand {
        VariableCommand {
            ident: self.transform_variable_name(&var.ident),
            var_type: var.var_type,
            input_comment: var.input_comment.clone(),
            max_length: var.max_length.as_ref().map(|s| self.transform_variable_name(s)).to_owned(),
        }
    }

    /// Escapes a variable name if it is contained in the vector of (disallowed)
    /// keywords.
    fn escape_keywords(&self, variable_name: String) -> String {
        // This is language dependent:
        // "string STRING" is valid cpp but "STRING : String" is not valid Pascal
        // even though the keyword "string" is expected to be escaped in both languages.
        let is_equal = if self.case_insensitive_keywords {
            str::eq_ignore_ascii_case
        } else {
            <str as PartialEq>::eq
        };

        if self.keywords.iter().any(|kw| is_equal(kw, &variable_name)) {
            format!("_{variable_name}")
        } else {
            variable_name
        }
    }

    fn convert(&self, variable_name: &str) -> String {
        match self.casing {
            Casing::SnakeCase => Self::convert_to_snake_case(variable_name),
            Casing::KebabCase => Self::convert_to_kebab_case(variable_name),
            Casing::PascalCase => Self::convert_to_pascal_case(variable_name),
            Casing::CamelCase => Self::convert_to_camel_case(variable_name),
        }
    }

    fn ident_words(ident: &str) -> Vec<String> {
        ident
            .chars()
            .peekable()
            .batching(|char_iter| {
                char_iter.peek()?; // check if there are any chars left

                // The word boundary seem to be non-lowercase characters in CG
                // Therefore we take
                // boundary characters + lowercase characters until next boundary
                let mut word_chars: Vec<char> =
                    char_iter.peeking_take_while(|c| !c.is_ascii_lowercase()).collect();
                word_chars.extend(char_iter.peeking_take_while(|c| c.is_ascii_lowercase()));

                Some(String::from_iter(word_chars).to_lowercase())
            })
            .collect()
    }

    fn convert_to_snake_case(variable_name: &str) -> String {
        Self::ident_words(variable_name).join("_")
    }

    fn convert_to_kebab_case(variable_name: &str) -> String {
        Self::ident_words(variable_name).join("-")
    }

    fn convert_to_pascal_case(variable_name: &str) -> String {
        variable_name[0..1].to_uppercase() + &variable_name[1..]
    }

    fn convert_to_camel_case(variable_name: &str) -> String {
        variable_name[0..1].to_lowercase() + &variable_name[1..]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake_case() {
        let convert = VariableNameOptions::convert_to_snake_case;
        assert_eq!("date_of_birth", convert("dateOfBirth"));
        assert_eq!("phrase_1braille_top_row", convert("Phrase1BrailleTopRow"));
        assert_eq!("crane_asciirepresentation", convert("craneASCIIRepresentation"));
    }

    #[test]
    fn test_kebab_case() {
        let convert = VariableNameOptions::convert_to_kebab_case;
        assert_eq!("date-of-birth", convert("dateOfBirth"));
        assert_eq!("phrase-1braille-top-row", convert("Phrase1BrailleTopRow"));
        assert_eq!("crane-asciirepresentation", convert("craneASCIIRepresentation"));
    }

    #[test]
    fn test_pascal_case() {
        let convert = VariableNameOptions::convert_to_pascal_case;
        assert_eq!("DateOfBirth", convert("dateOfBirth"));
        assert_eq!("Phrase1BrailleTopRow", convert("Phrase1BrailleTopRow"));
        assert_eq!("CraneASCIIRepresentation", convert("craneASCIIRepresentation"));
    }

    #[test]
    fn test_camel_case() {
        let convert = VariableNameOptions::convert_to_camel_case;
        assert_eq!("dateOfBirth", convert("dateOfBirth"));
        assert_eq!("phrase1BrailleTopRow", convert("Phrase1BrailleTopRow"));
        assert_eq!("craneASCIIRepresentation", convert("craneASCIIRepresentation"));
    }

    #[test]
    fn test_keywords_case_sensitive() {
        let variable_name_options = VariableNameOptions {
            casing: Casing::SnakeCase,
            allow_uppercase_vars: true,
            keywords: vec!["boolean".to_string()],
            case_insensitive_keywords: false,
        };

        // Does not change Boolean into _Boolean
        assert_eq!(variable_name_options.escape_keywords("Boolean".to_string()), "Boolean".to_string());
    }

    #[test]
    fn test_keywords_case_insensitive() {
        let variable_name_options = VariableNameOptions {
            casing: Casing::SnakeCase,
            allow_uppercase_vars: true,
            keywords: vec!["boolean".to_string()],
            case_insensitive_keywords: true,
        };

        // Changes Boolean into _Boolean
        assert_eq!(variable_name_options.escape_keywords("Boolean".to_string()), "_Boolean".to_string());
    }
}
