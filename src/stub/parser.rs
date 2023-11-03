use regex::Regex;

use super::{VariableStub, Stub};

pub struct Parser;

impl<'a> Parser {
    pub fn parse_read(mut stream: &mut impl Iterator<Item = &'a str>) -> Stub {
        Stub::Read(Self::parse_variable_list(&mut stream))
    }

    pub fn parse_write(stream: &mut impl Iterator<Item = &'a str>) -> Stub {
        let mut output: Vec<String> = Vec::new();

        while let Some(token) = stream.next() {
            let next_token = match token { 
                "\n" => {
                    match stream.next() {
                        Some("\n") | None => break,
                        Some(str) => format!("\n{}", str),
                    }
                }
                other => String::from(other),
            };

            output.push(next_token);
        };

        Stub::Write(output.join(" "))
    }

    pub fn parse_loop(mut stream: &mut impl Iterator<Item = &'a str>) -> Stub {
        let count = match stream.next() {
            Some("\n") | None => panic!("Loop stub not provided with loop count"),
            Some(other) => String::from(other),
        };

        let command = Box::new(Self::parse_read_or_write(&mut stream));

        Stub::Loop { count, command }
    }

    pub fn parse_loopline(mut stream: &mut impl Iterator<Item = &'a str>) -> Stub {
        let object = match stream.next() {
            Some("\n") | None => panic!("Loopline stub not provided with identifier to loop through"),
            Some(other) => String::from(other),
        };

        let variables = Self::parse_variable_list(&mut stream);

        Stub::LoopLine { object, variables }
    }

    fn parse_variable(token: &str) -> VariableStub {
        let mut iter = token.split(":");
        let identifier = String::from(iter.next().unwrap());
        let var_type = iter.next().expect("Error in stub generator: missing type");
        let length_regex = Regex::new(r"(word|string)\((\d+)\)").unwrap();
        let length_captures = length_regex.captures(var_type);
        match var_type {
            "int" => VariableStub::Int { name: identifier },
            "float" => VariableStub::Float { name: identifier },
            "long" => VariableStub::Long { name: identifier },
            "bool" => VariableStub::Bool { name: identifier },
            _ => {
                let caps = length_captures.expect("Failed to parse variable type in stub generator");
                let new_type = caps.get(1).unwrap().as_str();
                let var_length: usize = caps.get(2).unwrap().as_str().parse().unwrap();
                match new_type {
                    "word" => VariableStub::Word { name: identifier, max_length: var_length },
                    "string" => VariableStub::String { name: identifier, max_length: var_length },
                    _ => panic!("Unexpected error")
                }
            }
        }
    }

    fn parse_variable_list(stream: &mut impl Iterator<Item = &'a str>) -> Vec<VariableStub> {
        let mut vars = Vec::new();

        while let Some(token) = stream.next() {
            let var: VariableStub = match token {
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

    fn parse_read_or_write(mut stream: &mut impl Iterator<Item = &'a str>) -> Stub {
        match stream.next() {
            Some("read") => Parser::parse_read(&mut stream),
            Some("write") => Parser::parse_write(&mut stream),
            Some(thing) => panic!("Error parsing loop command in stub generator, got: {}", thing),
            None => panic!("Loop with no arguments in stub generator"),
        }
    }
}
