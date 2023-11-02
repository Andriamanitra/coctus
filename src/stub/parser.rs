use regex::Regex;

use super::ReadStub;

pub struct Parser;

impl<'a> Parser {
    pub fn parse_writes(stream: &mut impl Iterator<Item = &'a str>) -> String {
        let mut output = String::new();

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

            output.push_str(&next_token);
        };

        output
    }

    pub fn parse_readstub_list(stream: &mut impl Iterator<Item = &'a str>) -> Vec<ReadStub> {
        let mut reads = Vec::new();
        while let Some(token) = stream.next() {
            let read: ReadStub = match token {
                read if String::from(read).contains(":") => {
                    let mut iter = read.split(":");
                    let identifier = String::from(iter.next().unwrap());
                    let var_type = iter.next().expect("Error in stub generator: missing type");
                    let length_regex = Regex::new(r"(word|string)\((\d+)\)").unwrap();
                    let length_captures = length_regex.captures(var_type);
                    match var_type {
                        "int" => ReadStub::Int { name: identifier },
                        "float" => ReadStub::Float { name: identifier },
                        "long" => ReadStub::Long { name: identifier },
                        "bool" => ReadStub::Bool { name: identifier },
                        _ => {
                            let caps = length_captures.expect("Failed to parse read type in stub generator");
                            let new_type = caps.get(1).unwrap().as_str();
                            let var_length: usize = caps.get(2).unwrap().as_str().parse().unwrap();
                            match new_type {
                                "word" => ReadStub::Word { name: identifier, max_length: var_length },
                                "string" => ReadStub::String { name: identifier, max_length: var_length },
                                _ => panic!("Unexpected error")
                            }
                        }
                    }
                },
                "\n" => break,
                _ => panic!("Error in stub generator"),
            };

            reads.push(read);
        };
        reads
    }
}
