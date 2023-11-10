mod parser;
mod renderer;

use crate::programming_language::ProgrammingLanguage;

pub fn generate(lang: ProgrammingLanguage, generator: &str) -> String {
    let stub = parser::parse_generator_stub(generator.to_string());
    let output = renderer::render_stub(lang, stub.clone());

    format!("{}\n{:?}", output, stub)
}

