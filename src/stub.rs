mod parser;
mod renderer;

use anyhow::Result;

pub fn generate(lang: String, generator: &str) -> Result<String> {
    let stub = parser::parse_generator_stub(generator.to_string());
    let output_str = renderer::render_stub(&lang, stub.clone(), false)?;

    // eprint!("=======\n{:?}\n======", generator);
    eprint!("=======\n{}\n======\n", renderer::render_stub(&lang, stub.clone(), true)?);
    // eprint!("=======\n{:?}\n======", stub);

    Ok(output_str.as_str().trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_code_generation() {
        let generator = "read m:int n:int\nwrite result";
        let received = generate(String::from("ruby"), generator).unwrap();
        let expected = "m, n = gets.split.map(&:to_i)\nputs \"result\"";

        assert_eq!(received, expected);
    }
}
