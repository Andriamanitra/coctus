#![allow(clippy::while_let_on_iterator)]

use regex::Regex;

pub mod types;
pub use types::{Cmd, JoinTerm, JoinTermType, Stub, VariableCommand};

pub fn parse_generator_stub(generator: String) -> Stub {
    let generator = generator.replace('\n', " \n ");
    let stream = generator.split(' ');
    Parser::new(stream).parse()
}

struct Parser<StreamType: Iterator> {
    stream: StreamType,
}

impl<'a, I: Iterator<Item = &'a str>> Parser<I> {
    fn new(stream: I) -> Self {
        Self { stream }
    }

    #[rustfmt::skip]
    fn parse(&mut self) -> Stub {
        let mut stub = Stub::default();

        while let Some(token) = self.stream.next() {
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
        let mut write_text: Vec<String> = Vec::new();
        let mut first_line = true;

        while let Some(line) = self.upto_newline() {
            // NOTE: write•join()•rest⏎, with NOTHING inside the parens,
            //       gets parsed as a write and not as a write_join
            // NOTE: write•join("a")⏎ is a valid join
            // NOTE: write•join(⏎ gets parsed as a raw_string
            if let Some(position) = line
                .iter()
                .position(|&token| token.starts_with("join(") && !token.starts_with("join()") && first_line)
            {
                let result_slice = &line[position..];
                return self.parse_write_join(result_slice.to_vec())
            }
            first_line = false;
            write_text.push(line.join(" ").trim().to_string())
        }

        Cmd::Write {
            text: write_text.join("\n"),
            output_comment: String::new(),
        }
    }

    fn parse_write_join(&self, line_stream: Vec<&str>) -> Cmd {
        let inner = line_stream.join(" ");
        let terms_finder = Regex::new(r"join\(([^)]*)\)").unwrap();
        let terms_string_captures = terms_finder.captures(&inner);
        let terms_string = match terms_string_captures {
            None => {
                // in case write join(⏎
                return Cmd::Write {
                    text: inner,
                    output_comment: String::new(),
                }
            }
            Some(str) => str.get(1).unwrap().as_str(),
        };
        let term_splitter = Regex::new(r",\s*").unwrap();
        let literal_matcher = Regex::new("\\\"([^)]+)\\\"").unwrap();

        let join_terms = term_splitter
            .split(terms_string)
            .map(|term_str| {
                if let Some(mtch) = literal_matcher.captures(term_str) {
                    JoinTerm::new(mtch.get(1).unwrap().as_str().to_owned(), JoinTermType::Literal)
                } else {
                    JoinTerm::new(term_str.to_owned(), JoinTermType::Variable)
                }
            })
            .collect::<Vec<_>>();

        // write•join("hi",,,•"Jim")⏎ should be rendered as a Write Cmd
        // (I guess the CG parser fails due to consecutive commas)
        if join_terms.iter().any(|jt| jt.name.is_empty()) {
            return Cmd::Write {
                text: inner,
                output_comment: String::new(),
            }
        }

        Cmd::WriteJoin {
            join_terms,
            output_comment: String::new(),
        }
    }

    fn parse_loop(&mut self) -> Cmd {
        match self.next_past_newline() {
            Some("\n") => panic!("Could not find count identifier for loop"),
            None => panic!("Unexpected end of input: Loop stub not provided with loop count"),
            Some(other) => Cmd::Loop {
                count_var: String::from(other),
                command: Box::new(self.parse_loopable()),
            },
        }
    }

    fn parse_loopable(&mut self) -> Cmd {
        match self.next_past_newline() {
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
        match self.next_past_newline() {
            Some("\n") => panic!("Could not find count identifier for loopline"),
            None => panic!("Unexpected end of input: Loopline stub not provided with count identifier"),
            Some(other) => Cmd::LoopLine {
                count_var: other.to_string(),
                variables: self.parse_variables(),
            },
        }
    }

    fn parse_variable(token: &str) -> VariableCommand {
        let mut iter = token.split(':');
        let identifier = String::from(iter.next().unwrap());
        let var_type = iter.next().expect("Error in stub generator: missing type");

        // Trim because the stub generator may contain sneaky newlines
        match var_type.trim_end() {
            "int" => VariableCommand::new(identifier, types::VarType::Int, None),
            "float" => VariableCommand::new(identifier, types::VarType::Float, None),
            "long" => VariableCommand::new(identifier, types::VarType::Long, None),
            "bool" => VariableCommand::new(identifier, types::VarType::Bool, None),
            _ => {
                let length_regex = Regex::new(r"(word|string)\((\w+)\)").unwrap();
                let length_captures = length_regex.captures(var_type);
                let caps = length_captures
                    .unwrap_or_else(|| panic!("Failed to parse variable type for token: {}", &token));
                let new_type = caps.get(1).unwrap().as_str();
                let length = caps.get(2).unwrap().as_str();
                let max_length = String::from(length);
                match new_type {
                    "word" => VariableCommand::new(identifier, types::VarType::Word, Some(max_length)),
                    "string" => VariableCommand::new(identifier, types::VarType::String, Some(max_length)),
                    _ => panic!("Unexpected error"),
                }
            }
        }
    }

    fn parse_variables(&mut self) -> Vec<VariableCommand> {
        let mut vars = Vec::new();
        let Some(line) = self.upto_newline() else {
            panic!("Empty line after read keyword")
        };

        for token in line {
            if !token.is_empty() {
                vars.push(Self::parse_variable(token))
            }
        }

        vars
    }

    fn parse_output_comment(&mut self, previous_commands: &mut [Cmd]) {
        let output_comment = self.parse_text_block();
        for cmd in previous_commands {
            Self::update_cmd_with_output_comment(cmd, &output_comment)
        }
    }

    fn update_cmd_with_output_comment(cmd: &mut Cmd, new_comment: &str) {
        match cmd {
            Cmd::Write {
                text: _,
                ref mut output_comment,
            }
            | Cmd::WriteJoin {
                join_terms: _,
                ref mut output_comment,
            } if output_comment.is_empty() => *output_comment = new_comment.to_string(),
            Cmd::Loop {
                count_var: _,
                ref mut command,
            } => {
                Self::update_cmd_with_output_comment(command, new_comment);
            }
            _ => (),
        }
    }

    // Doesn't deal with InputComments to unassigned variables
    // nor InputComments to variables with the same identifier
    fn parse_input_comment(&mut self, previous_commands: &mut [Cmd]) {
        let input_statement = self.parse_text_block();
        let input_comments = input_statement
            .lines()
            .filter(|line| line.contains(':'))
            .map(|line| {
                if let Some((var, rest)) = line.split_once(':') {
                    (String::from(var.trim()), String::from(rest.trim()))
                } else {
                    panic!("Impossible since the list was filtered??");
                }
            })
            .collect::<Vec<_>>();

        for (ic_ident, ic_comment) in input_comments {
            for cmd in previous_commands.iter_mut() {
                Self::update_cmd_with_input_comment(cmd, &ic_ident, &ic_comment);
            }
        }
    }

    fn update_cmd_with_input_comment(cmd: &mut Cmd, ic_ident: &String, ic_comment: &String) {
        match cmd {
            Cmd::Read(variables)
            | Cmd::LoopLine {
                count_var: _,
                variables,
            } => {
                for var in variables.iter_mut() {
                    if var.ident == *ic_ident {
                        var.input_comment = ic_comment.clone();
                    }
                }
            }
            Cmd::Loop {
                count_var: _,
                ref mut command,
            } => {
                Self::update_cmd_with_input_comment(command, ic_ident, ic_comment);
            }
            _ => (),
        }
    }

    fn skip_to_next_line(&mut self) {
        while let Some(token) = self.stream.next() {
            if token == "\n" {
                break
            }
        }
    }

    fn parse_text_block(&mut self) -> String {
        self.skip_to_next_line();

        let mut text_block: Vec<String> = Vec::new();
        while let Some(line) = self.upto_newline() {
            text_block.push(line.join(" ").trim().to_string())
        }

        text_block.join("\n")
    }

    fn next_past_newline(&mut self) -> Option<&'a str> {
        match self.stream.next() {
            Some("\n") => self.stream.next(),
            Some("") => self.next_past_newline(),
            token => token,
        }
    }

    // Consumes the newline
    fn upto_newline(&mut self) -> Option<Vec<&'a str>> {
        let mut buf = Vec::new();
        while let Some(token) = self.stream.next() {
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
