#![allow(clippy::while_let_on_iterator)]

use super::{Cmd, JoinTerm, Stub, VariableCommand, VarType};
use std::iter;

pub fn parse_generator_stub(generator: &str) -> Stub {
    Parser::new(generator).parse()
}

/// A wrapper around an iterator of tokens in the CG stub. Contains all of the stub parsing logic.
///
/// Exists solely to be consumed with `.parse()`
struct Parser<'a> {
    token_stream: Box<dyn Iterator<Item = &'a str> + 'a>,
}

impl<'a> Parser<'a> {
    fn new(stub: &'a str) -> Self {
        // .chain just adds an iterator to the end of another one,
        // iter::once creates an iterator out of a single element. 
        // Essentially this puts a "\n" at the end of each line so the parser can tell where the
        // lines end. Unfortunately I cannot concat &strs which would have made this much simpler.
        let token_stream = stub.lines().flat_map(|line| line.split(' ').chain(iter::once("\n")));
        Self { token_stream: Box::new(token_stream) }
    }

    #[rustfmt::skip]
    fn parse(mut self) -> Stub {
        let mut stub = Stub::default();

        while let Some(token) = self.next_token() {
            match token {
                "read"      => stub.commands.push(self.parse_read()),
                "write"     => stub.commands.push(self.parse_write()),
                "loop"      => stub.commands.push(self.parse_loop()),
                "loopline"  => stub.commands.push(self.parse_loopline()),
                "OUTPUT"    => self.parse_output_comment(&mut stub.commands),
                "INPUT"     => self.parse_input_comment(&mut stub.commands),
                "STATEMENT" => stub.statement = self.parse_text_block(),
                "\n" | ""   => continue,
                thing => panic!("Unknown token stub generator: '{}'", thing),
            };
        }

        stub
    }

    fn parse_read(&mut self) -> Cmd {
        Cmd::Read(self.parse_variables())
    }

    fn parse_write(&mut self) -> Cmd {
        let mut lines = Vec::new();

        while let Some(line) = self.rest_of_line() {
            // NOTE: A join could be present on the first line
            if lines.is_empty() {
                if let Some(write) = self.check_for_write_join(&line) {
                    return write
                }
            }

            lines.push(line)
        }

        Cmd::Write {
            lines,
            output_comment: Vec::new(),
        }
    }

    fn check_for_write_join(&self, line: &str) -> Option<Cmd> {
        // NOTE: write•join()•rest⏎, with NOTHING inside the parens,
        //       gets parsed as a write and not as a write_join
        match line.replace("join()", "").split_once("join(") {
            Some((_, join_arg)) if join_arg.contains(")") => {
                let terms_string = join_arg.split_once(")").expect("Already checked existence").0;

                if terms_string.split(",").any(|t| t.trim().is_empty()) {
                    // write•join("hi",,,•"Jim")⏎ should be rendered as a Write Cmd
                    // (I guess the CG parser fails due to consecutive commas)
                    Some(Cmd::Write { 
                            lines: vec![line.to_string()], 
                            output_comment: Vec::new() 
                        })
                } else {
                    // NOTE: write•join("a")⏎ is a valid join
                    Some(self.parse_write_join(terms_string))
                }
            }
            // NOTE: write•join(⏎ gets parsed as a raw string
            //       and write parsing resumes
            _ => None
        }
    }

    fn parse_write_join(&self, terms_string: &str) -> Cmd {
        let join_terms =  
            terms_string.split(",").map(|term|
                if term.contains('"') {
                    let term_name = term.trim_matches(|c| c != '"').trim_matches('"').to_string();
                    JoinTerm::new_literal(term_name)
                } else { 
                    JoinTerm::new_variable(term.trim().to_string())
                }
            ).collect();

        Cmd::WriteJoin { 
            join_terms,
            output_comment: Vec::new() 
        }
    }

    fn parse_loop(&mut self) -> Cmd {
        match self.next_token_past_newline() {
            Some("\n") => panic!("Could not find count identifier for loop"),
            None => panic!("Unexpected end of input: Loop stub not provided with loop count"),
            Some(other) => Cmd::Loop {
                count_var: String::from(other),
                command: Box::new(self.parse_loopable()),
            },
        }
    }

    fn parse_loopable(&mut self) -> Cmd {
        match self.next_token_past_newline() {
            Some("\n") => panic!("Loop not provided with command"),
            Some("read") => self.parse_read(),
            Some("write") => self.parse_write(),
            Some("loopline") => self.parse_loopline(),
            Some("loop") => self.parse_loop(),
            Some(thing) => panic!("Error parsing loop command in stub generator, got: {}", thing),
            None => panic!("Unexpected end of input, expecting command to loop through"),
        }
    }

    fn parse_loopline(&mut self) -> Cmd {
        match self.next_token_past_newline() {
            Some("\n") => panic!("Could not find count identifier for loopline"),
            None => panic!("Unexpected end of input: Loopline stub not provided with count identifier"),
            Some(other) => Cmd::LoopLine {
                count_var: other.to_string(),
                variables: self.parse_variables(),
            },
        }
    }

    fn parse_variables(&mut self) -> Vec<VariableCommand> {
        let Some(tokens) = self.tokens_upto_newline() else {
            panic!("Empty line after read keyword")
        };

        tokens.into_iter().filter_map(Self::parse_variable).collect()
    }

    fn parse_variable(token: &str) -> Option<VariableCommand> {
        if token.is_empty() { return None }
        let Some((ident, type_string)) = token.split_once(':') else { panic!("Variable must have type") };
        let (var_type, max_length) = Self::extract_type_and_length(type_string);

        Some(VariableCommand::new(ident.to_string(), var_type, max_length))
    }

    fn extract_type_and_length(type_string: &str) -> (VarType, Option<String>) {
        match type_string.trim_end_matches(')').split_once('(') {
            None => (VarType::new_unsized(type_string), None),
            Some((var_type, max_length)) => (VarType::new_sized(var_type), Some(max_length.to_string()))
        }
    }

    fn parse_output_comment(&mut self, previous_commands: &mut [Cmd]) {
        let output_comment = self.parse_text_block();
        for cmd in previous_commands {
            Self::update_cmd_with_output_comment(cmd, &output_comment)
        }
    }

    // Doesn't deal with InputComments to unassigned variables
    // nor InputComments to variables with the same identifier
    fn parse_input_comment(&mut self, previous_commands: &mut [Cmd]) {
        self.skip_to_next_line();

        while let Some(line) = self.rest_of_line() {
            if let Some((ic_ident, ic_comment)) = line.split_once(':') {
                for cmd in previous_commands.iter_mut() {
                    Self::update_cmd_with_input_comment(cmd, ic_ident.trim(), ic_comment.trim());
                }
            }
        }
    }

    fn update_cmd_with_output_comment(cmd: &mut Cmd, new_comment: &Vec<String>) {
        match cmd {
            Cmd::Write { ref mut output_comment, .. } | Cmd::WriteJoin { ref mut output_comment, .. } 
                if output_comment.is_empty() => *output_comment = new_comment.clone(),
            Cmd::Loop { ref mut command, .. } => {
                Self::update_cmd_with_output_comment(command, new_comment);
            }
            _ => (),
        }
    }

    fn update_cmd_with_input_comment(cmd: &mut Cmd, ic_ident: &str, ic_comment: &str) {
        match cmd {
            Cmd::Read(variables) | Cmd::LoopLine { variables, .. } => {
                for var in variables.iter_mut().filter(|var| var.ident == *ic_ident) {
                    var.input_comment = ic_comment.to_string();
                }
            }
            Cmd::Loop { ref mut command, .. } => {
                Self::update_cmd_with_input_comment(command, ic_ident, ic_comment);
            }
            _ => (),
        }
    }

    fn skip_to_next_line(&mut self) {
        while let Some(token) = self.next_token() {
            if token == "\n" {
                break
            }
        }
    }

    fn parse_text_block(&mut self) -> Vec<String> {
        self.skip_to_next_line();

        let mut text_block = Vec::new();

        while let Some(line) = self.rest_of_line() {
            text_block.push(line.trim().to_string())
        }

        text_block
    }

    fn next_token(&mut self) -> Option<&'a str> {
        self.token_stream.next()
    }

    fn next_token_past_newline(&mut self) -> Option<&'a str> {
        match self.next_token() {
            Some("\n") => self.next_token(),
            Some("") => self.next_token_past_newline(),
            token => token,
        }
    }

    fn rest_of_line(&mut self) -> Option<String> {
        Some(self.tokens_upto_newline()?.join(" ").trim().to_string())
    }

    // Consumes the newline
    fn tokens_upto_newline(&mut self) -> Option<Vec<&'a str>> {
        let mut buf = Vec::new();

        while let Some(token) = self.next_token() {
            if token == "\n" {
                break
            }
            buf.push(token)
        }

        if buf.join("").is_empty() {
            None
        } else {
            Some(buf)
        }
    }

}

#[cfg(test)]
mod parser_tests;
