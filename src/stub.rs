mod parser;
pub mod renderer;

use anyhow::Result;
use renderer::language::Language;

pub fn generate(lang: Language, generator: &str) -> Result<String> {
    let stub = parser::parse_generator_stub(generator);

    // eprint!("=======\n{:?}\n======\n", generator);
    eprint!("=======\n{}\n======\n", renderer::render_stub(lang.clone(), stub.clone(), true)?);
    // eprint!("=======\n{:?}\n======\n", stub);

    let output_str = renderer::render_stub(lang.clone(), stub, false)?;

    Ok(output_str.as_str().trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_code_generation() {
        let lang = Language::try_from("ruby").unwrap();
        let generator = "read m:int n:int\nwrite result";
        let received = generate(lang, generator).unwrap();
        let expected = "m, n = gets.split.map(&:to_i)\nputs \"result\"";

        assert_eq!(received, expected);
    }

    const REFERENCE_STUB: &str = r##"write many  spaces   here

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
write join("hello", a, "planet")"##;

    #[test]
    fn test_reference_stub_ruby() {
        let lang = Language::try_from("ruby").unwrap();
        let received = generate(lang, REFERENCE_STUB).unwrap();
        let expected = r##"# Live long
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
puts "hello #{a} planet""##;

        for (r, e) in received.lines().zip(expected.lines()) {
            assert_eq!(r, e)
        }
    }

    // Just test that it compiles
    #[test]
    fn test_reference_stub_rust() {
        let lang = Language::try_from("rust").unwrap();
        generate(lang, REFERENCE_STUB).unwrap();
    }

    #[test]
    fn test_reference_stub_c() {
        let lang = Language::try_from("C").unwrap();
        generate(lang, REFERENCE_STUB).unwrap();
    }
}
