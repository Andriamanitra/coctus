mod parser;
mod renderer;

pub fn generate(lang: String, generator: &str) -> String {
    let stub = parser::parse_generator_stub(generator.to_string());
    let output_str = renderer::render_stub(lang, stub.clone());

    format!("At stub::generate:\n{}\n{:?}", output_str, stub)
}
