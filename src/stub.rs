pub mod language;
mod parser;
mod renderer;
pub mod stub_config;
pub mod preprocessor;

use anyhow::Result;
use indoc::indoc;
pub use language::Language;
use serde::Serialize;
pub use stub_config::StubConfig;

use preprocessor::RenderableCmd;

pub fn generate(config: StubConfig, generator: &str) -> Result<String> {
    let stub = parser::parse_generator_stub(generator)?;

    // eprint!("=======\n{:?}\n======\n", generator);
    // eprint!("=======\n{:?}\n======\n", stub);

    let output_str = renderer::render_stub(config.clone(), stub)?;

    Ok(output_str.as_str().trim().to_string())
}

#[derive(Clone, Default)]
pub struct Stub {
    pub commands: Vec<Cmd>,
    pub statement: Vec<String>,
}

// More visual than derive(Debug)
impl std::fmt::Debug for Stub {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Stub {{\n  commands: [")?;
        for command in &self.commands {
            write!(f, "\n    {:?}", command)?;
        }
        write!(f, "\n  ],\n  statement: {:?}\n}}", self.statement)
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Hash)]
pub enum VarType {
    Int,
    Float,
    Long,
    Bool,
    Word,
    String,
}

impl<'a> VarType {
    fn new_unsized(value: &'a str) -> Self {
        match value {
            "int" => VarType::Int,
            "float" => VarType::Float,
            "long" => VarType::Long,
            "bool" => VarType::Bool,
            other => panic!("No unsized variable type: {other}"),
        }
    }

    fn new_sized(value: &'a str) -> Self {
        match value {
            "word" => VarType::Word,
            "string" => VarType::String,
            other => panic!("No sized variable type: {other}"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct VariableCommand {
    pub ident: String,
    pub var_type: VarType,
    pub max_length: Option<String>,
    pub input_comment: String,
}

impl VariableCommand {
    pub fn new(ident: String, var_type: VarType, max_length: Option<String>) -> VariableCommand {
        VariableCommand {
            ident,
            var_type,
            max_length,
            input_comment: String::new(),
        }
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct JoinTerm {
    pub ident: String,
    pub var_type: Option<VarType>,
}

impl JoinTerm {
    pub fn new(ident: String, var_type: Option<VarType>) -> JoinTerm {
        JoinTerm { ident, var_type }
    }
}

#[derive(Debug, Clone)]
pub enum Cmd {
    Read(Vec<VariableCommand>),
    Loop {
        count_var: String,
        command: Box<Cmd>,
    },
    LoopLine {
        count_var: String,
        variables: Vec<VariableCommand>,
    },
    Write {
        lines: Vec<String>,
        output_comment: Vec<String>,
    },
    WriteJoin {
        join_terms: Vec<JoinTerm>,
        output_comment: Vec<String>,
    },
    External(Box<dyn RenderableCmd>),
}

pub const SIMPLE_REFERENCE_STUB: &str = indoc! {r##"
    read anInt:int
    read aFloat:float
    read Long:long
    read aWord:word(1)
    read boolean:bool
    read ABC1ABc1aBC1AbC1abc1:int
    read STRING:string(256)
    read anInt2:int aFloat2:float Long2:long aWord2:word(1) boolean2:bool
    loop anInt read x:int
    loop anInt read x:int f:float
    loop anInt loop anInt read x:int y:int
    loopline anInt x:int
    loopline anInt w:word(50)
    loopline anInt x:int f:float w:word(50)
    write result

    OUTPUT
    An output comment

    write join(anInt, aFloat, Long, boolean)

    write join(aWord, "literal", STRING)

    STATEMENT
    This is the statement

    INPUT
    anInt: An input comment over anInt
"##};

#[cfg(test)]
mod tests {
    use super::*;

    const COMPLEX_REFERENCE_STUB: &str = indoc! {r##"
        write many  spaces   here

        read try:bool
        read nil:string(50)
        read L:string(20)

        OUTPUT
        The spacemaster


        INPUT
        a: does stuff

        read a:word(50) b:word(50)
        read xTra:int y:int
        read annoying:word(xTra)
        read anotherAnnoying:word(y)
        read aBc:string(256)
        read ROW:string(1024)

        INPUT
        ROW: Your boat
        This is ignored
        aBc: The alphabet

        loop N read EXT:word(100) MT:word(100)
        loop N read count:int name:word(50)

        loop Q read FNAME:string(500)

        loop 4 read number:int

        loop 4 write 0 0

        read n:int
        loop 
        n    
        loop 4
        write thing
        write thing

        write thing

        read xCount:int
        loopline xCount x:int
        loopline xCount x:int y:int z:word(50)
        STATEMENT junk
        Live long
        and prosper
              and a line with spaces both sides   

        write something something join(a, b)
        write something join(a, b) something
        write join(a, "b", aBc)
        write join("hello", "world")
        write join("hello", a, "planet")
    "##};

    #[test]
    fn test_simple_code_generation() {
        let cfg = StubConfig::read_from_embedded("ruby").unwrap();
        let generator = "read m:int n:int\nwrite result";
        let received = generate(cfg, generator).unwrap();
        let expected = "m, n = gets.split.map(&:to_i)\nputs \"result\"";

        assert_eq!(received, expected);
    }

    #[test]
    fn test_reference_stub_ruby() {
        let cfg = StubConfig::read_from_embedded("ruby").unwrap();
        let received = generate(cfg, COMPLEX_REFERENCE_STUB).unwrap();
        let expected = indoc! { r##"
            # Live long
            # and prosper
            # and a line with spaces both sides

            # The spacemaster
            puts "many  spaces   here"
            try = gets.to_bool
            _nil = gets.chomp
            l = gets.chomp
            a, b = gets.split
            x_tra, y = gets.split.map(&:to_i)
            annoying = gets
            another_annoying = gets
            a_bc = gets.chomp # The alphabet
            row = gets.chomp # Your boat
            n.times do
              ext, mt = gets.split
            end
            n.times do
              count, name = gets.split
              count = count.to_i
              name = name.chomp
            end
            q.times do
              fname = gets.chomp
            end
            4.times do
              number = gets.to_i
            end
            4.times do
              puts "0 0"
            end
            n = gets.to_i
            n.times do
              4.times do
                puts "thing"
                puts "write thing"
              end
            end
            puts "thing"
            x_count = gets.to_i
            gets.split.each do |x|
              x = x.to_i
            end
            gets.split.each_slice(3) do |x, y, z|
              x = x.to_i
              y = y.to_i
            end
            puts "#{a} #{b}"
            puts "#{a} #{b}"
            puts "#{a} b #{a_bc}"
            puts "hello world"
            puts "hello #{a} planet"
        "##};

        for (r, e) in received.lines().zip(expected.lines()) {
            assert_eq!(r, e)
        }
    }

    // Just test that it compiles
    #[test]
    fn test_reference_stub_rust() {
        let cfg = StubConfig::read_from_embedded("rust").unwrap();
        generate(cfg, COMPLEX_REFERENCE_STUB).unwrap();
    }

    #[test]
    fn test_reference_stub_c() {
        let cfg = StubConfig::read_from_embedded("c").unwrap();
        generate(cfg, COMPLEX_REFERENCE_STUB).unwrap();
    }

    #[test]
    fn test_reference_stub_cpp() {
        let cfg = StubConfig::read_from_embedded("cpp").unwrap();
        generate(cfg, COMPLEX_REFERENCE_STUB).unwrap();
    }
}
