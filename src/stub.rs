mod parser;

use parser::Parser;
use crate::programming_language::ProgrammingLanguage;

pub fn generate(lang: ProgrammingLanguage, generator: &str) -> String {
    let binding = generator.replace("\n", " \n ").replace("\n  \n", "\n \n");
    let mut stream = binding.split(" ");
    let mut stub_parts: Vec<Stub> = Vec::new();

    while let Some(token) = stream.next() {
        let stub_part = match token {
            "read" => Stub::Read(Parser::parse_readstub_list(&mut stream)),
            "write" => Stub::Write(Parser::parse_writes(&mut stream)),
            // TODO: Add loop and loopline
            "\n" | "" => continue,
            thing => panic!("Error parsing stub generator: {}", thing),
        };
        stub_parts.push(stub_part);
    }

    format!("{:?}", stub_parts)
}

#[derive(Debug)]
pub enum ReadStub {
  Int { name: String },
  Float { name: String },
  Long { name: String },
  Bool { name: String },
  Word { name: String, max_length: usize },
  String { name: String, max_length: usize },
}

#[derive(Debug)]
enum Stub {
  Read(Vec<ReadStub>),
  Loop(Box<Stub>),
  LoopLine(Box<Stub>),
  Write(String),
}
