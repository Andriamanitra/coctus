mod parser;

use parser::Parser;
use crate::programming_language::ProgrammingLanguage;

pub fn generate(lang: ProgrammingLanguage, generator: &str) -> String {
    let binding = generator.replace("\n", " \n ").replace("\n  \n", "\n \n");
    let mut stream = binding.split(" ");
    let mut stub_parts: Vec<Stub> = Vec::new();

    while let Some(token) = stream.next() {
        let stub_part = match token {
            // TODO: Add loop and loopline
            "read" => Parser::parse_read(&mut stream),
            "write" => Parser::parse_write(&mut stream),
            "\n" | "" => continue,
            thing => panic!("Error parsing stub generator: {}", thing),
        };
        stub_parts.push(stub_part);
    }

    format!("{:?}", stub_parts)
}

#[derive(Debug)]
pub enum VariableStub {
  Int { name: String },
  Float { name: String },
  Long { name: String },
  Bool { name: String },
  Word { name: String, max_length: usize },
  String { name: String, max_length: usize },
}

#[derive(Debug)]
pub enum Stub {
  Read(Vec<VariableStub>),
  Loop { count: String, command: Box<Stub> },
  LoopLine { object: String, variables: Vec<VariableStub> },
  Write(String),
}
