mod parser;
mod renderer;

pub fn generate(lang: String, generator: &str) -> String {
    let stub = parser::parse_generator_stub(generator.to_string());
    let output_str = renderer::render_stub(&lang, stub.clone(), false);

    // eprint!("=======\n{:?}\n======", generator);
    eprint!("=======\n{}\n======", renderer::render_stub(&lang, stub.clone(), true));
    // eprint!("=======\n{:?}\n======", stub);

    output_str.as_str().trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_code_generation() {
        let generator = "read m:int n:int\nwrite result";
        let received = generate(String::from("ruby"), generator);
        let expected = "m, n = gets.split.map(&:to_i)\nputs \"result\"";

        assert_eq!(received, expected);
    }
}
