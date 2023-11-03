use regex::Regex;

use super::{VariableCommand, Command, Stub, InputComment};


pub fn parse_generator_stub(generator: String) -> Stub {
    let generator = generator.replace("\n", " \n ").replace("\n  \n", "\n \n");
    let stream = generator.split(" ");
    Parser::new(stream).parse()
}

struct Parser<StreamType: Iterator> {
    stream: StreamType,
}

impl<'a, I: Iterator<Item = &'a str>> Parser<I> {
    fn new(stream: I) -> Self {
        Self { stream }
    }

    fn parse(&mut self) -> Stub {
        let mut stub = Stub::new();

        while let Some(token) = self.stream.next() {
            match token {
                "read" => stub.commands.push(self.parse_read()),
                "write" => stub.commands.push(self.parse_write()),
                "loop" => stub.commands.push(self.parse_loop()),
                "loopline" => stub.commands.push(self.parse_loopline()),
                "OUTPUT" => stub.output_comment = self.parse_output_comment(),
                "INPUT" => stub.input_comments.append(&mut self.parse_input_comment()),
                "STATEMENT" => stub.statement = self.parse_statement(),
                "\n" | "" => continue,
                thing => panic!("Error parsing stub generator: {}", thing),
            };
        }

        stub
    }

    fn parse_read(&mut self) -> Command {
        Command::Read(self.parse_variable_list())
    }

    fn parse_write(&mut self) -> Command {
        let mut output: Vec<String> = Vec::new();

        while let Some(token) = self.stream.next() {
            let next_token = match token { 
                "\n" => {
                    match self.stream.next() {
                        Some("\n") | None => break,
                        Some(str) => format!("\n{}", str),
                    }
                }
                other => String::from(other),
            };

            output.push(next_token);
        };

        Command::Write(output.join(" "))
    }

    fn parse_loop(&mut self) -> Command {
        let count = match self.stream.next() {
            Some("\n") | None => panic!("Loop stub not provided with loop count"),
            Some(other) => String::from(other),
        };

        let command = Box::new(self.parse_read_or_write());

        Command::Loop { count, command }
    }

    fn parse_loopline(&mut self) -> Command {
        let object = match self.stream.next() {
            Some("\n") | None => panic!("Loopline stub not provided with identifier to loop through"),
            Some(other) => String::from(other),
        };

        let variables = self.parse_variable_list();

        Command::LoopLine { object, variables }
    }

    fn parse_variable(token: &str) -> VariableCommand {
        let mut iter = token.split(":");
        let identifier = String::from(iter.next().unwrap());
        let var_type = iter.next().expect("Error in stub generator: missing type");
        let length_regex = Regex::new(r"(word|string)\((\d+)\)").unwrap();
        let length_captures = length_regex.captures(var_type);
        match var_type {
            "int" => VariableCommand::Int { name: identifier },
            "float" => VariableCommand::Float { name: identifier },
            "long" => VariableCommand::Long { name: identifier },
            "bool" => VariableCommand::Bool { name: identifier },
            _ => {
                let caps = length_captures.expect("Failed to parse variable type in stub generator");
                let new_type = caps.get(1).unwrap().as_str();
                let var_length: usize = caps.get(2).unwrap().as_str().parse().unwrap();
                match new_type {
                    "word" => VariableCommand::Word { name: identifier, max_length: var_length },
                    "string" => VariableCommand::String { name: identifier, max_length: var_length },
                    _ => panic!("Unexpected error")
                }
            }
        }
    }

    fn parse_variable_list(&mut self) -> Vec<VariableCommand> {
        let mut vars = Vec::new();

        while let Some(token) = self.stream.next() {
            let var: VariableCommand = match token {
                _ if String::from(token).contains(":") => {
                    Self::parse_variable(token)
                },
                "\n" => break,
                unexp => panic!("Error in stub generator, found {unexp} while searching for stub variables"),
            };

            vars.push(var);
        };

        vars
    }

    fn parse_read_or_write(&mut self) -> Command {
        match self.stream.next() {
            Some("read") => self.parse_read(),
            Some("write") => self.parse_write(),
            Some(thing) => panic!("Error parsing loop command in stub generator, got: {}", thing),
            None => panic!("Loop with no arguments in stub generator"),
        }
    }

    fn parse_output_comment(&self) -> String {
        todo!()
    }

    fn parse_input_comment(&self) -> Vec<InputComment> {
        todo!()
    }

    fn parse_statement(&self) -> String {
        todo!()
    }
}
