use clashlib::stub::{self, StubConfig};

fn test_stub_builder(generator: &str, expected: &str) {
    let cfg = StubConfig::read_from_embedded("python").unwrap();
    let received = stub::generate(cfg, generator).unwrap().as_str().trim().to_string();
    let expected = expected.trim();

    assert_eq!(expected.lines().count(), received.lines().count());
    for (r, e) in expected.lines().zip(received.lines()) {
        assert_eq!(r, e)
    }
}

#[test]
fn test_stub_read_1() {
    let generator = r##"read anInt:int
read I:int
read aFloat:float
read aLong:long
read aString:string(256)
read aWord:word(256)
read    Spaces:string(10)     
read aBOOL:bool
"##;
    let expected = r##"an_int = int(input())
i = int(input())
a_float = float(input())
a_long = int(input())
a_string = input()
a_word = input()
spaces = input()
a_bool = input() != "0"
"##;

    test_stub_builder(generator, expected);
}

#[test]
fn test_stub_read_2() {
    let generator = r##"read x:int y:int
read x:int y:float
read two:word(50) words:word(50)
read aWord:word(50) x:int
read inputs:string(256)
"##;
    let expected = r##"x, y = [int(i) for i in input().split()]
inputs = input().split()
x = int(inputs[0])
y = float(inputs[1])
two, words = input().split()
inputs = input().split()
a_word = inputs[0]
x = int(inputs[1])
inputs = input()
"##;

    test_stub_builder(generator, expected);
}

#[test]
fn test_stub_loop() {
    let generator = r##"read nLoop:int
loop nLoop read anInt:int
loop nLoop read anInt:int aFloat:float aWord:word(256)
read nLoopLines:int
loopline nLoopLines anInt:int
loopline nLoopLines aFloat:float aLong:long
loop nLoop loopline 5 word:word(1)
loop nLoop loopline 5 string:string(1)
"##;
    let expected = r##"n_loop = int(input())
for i in range(n_loop):
    an_int = int(input())
for i in range(n_loop):
    inputs = input().split()
    an_int = int(inputs[0])
    a_float = float(inputs[1])
    a_word = inputs[2]
n_loop_lines = int(input())
for i in input().split():
    an_int = int(i)
inputs = input().split()
for i in range(n_loop_lines):
    a_float = float(inputs[2*i])
    a_long = int(inputs[2*i+1])
for i in range(n_loop):
    for word in input().split():
        pass
for i in range(n_loop):
    for j in input().split():
        string = j
"##;

    test_stub_builder(generator, expected);
}

#[test]
fn test_stub_write_1() {
    let generator = r##"write Never

read n:int
loop n write gonna

loop n loop n write let

write you
down

write write

write    care, here   spaces   everywhere    
    and some  more   

write and dont do this, this breaks paitong

write "
'
"
"##;
    let expected = r##"print("Never")
n = int(input())
for i in range(n):
    print("gonna")
for i in range(n):
    for j in range(n):
        print("let")
print("you")
print("down")
print("write")
print("care, here   spaces   everywhere")
print("and some  more")
print("and dont do this, this breaks paitong")
print(""")
print("'")
print(""")
"##;

    test_stub_builder(generator, expected);
}

#[test]
fn test_stub_write_2() {
    let generator = r##"read n:int
read aBc:int

write join("a", "b")
write join("a", n)
write join(n, "a")
write join(n, aBc)

write THIS IS IGNORED join(n, aBc)
write something join(n, aBc, n) something

write join(n, "potato", aBc, n)
"##;
    let expected = r##"n = int(input())
a_bc = int(input())
print("a b")
print("a " + str(n))
print(str(n) + " a")
print(str(n) + " " + str(a_bc))
print(str(n) + " " + str(a_bc))
print(str(n) + " " + str(a_bc) + " " + str(n))
print(str(n) + " potato " + str(a_bc) + " " + str(n))

"##;

    test_stub_builder(generator, expected);
}

#[test]
fn test_stub_write_3() {
    let generator = r##"read n:int
read aBc:int
write join(  "  le  "  ,  " spaces  "  )

write join(

write join (baited)
This only works because the next join gets parsed as raw text 
write join(       ) 

write join("a")
write join("b") join("IGNORED")
write join("c") join(

write join(join("a"), n, join("b")))))

write join("d", write("writeception"))
write join("d", write(join("a", aBc)))
write join("d", write join("a", aBc) )
"##;
    let expected = r##"n = int(input())
a_bc = int(input())
print("  le    spaces  ")
print("join(")
print("join (baited)")
print("This only works because the next join gets parsed as raw text")
print("write join(       )")
print("a")
print("b")
print("c")
print("a")
print("d writeception")
print("d a " + str(a_bc))
print("d a " + str(a_bc))

"##;

    test_stub_builder(generator, expected);
}

#[test]
fn test_stub_write_4() {
    let generator = r##"read NONSENSE:int
write join("hi", NONSENSE INJECTED "it's me Jim" WHAT THE F$)
write join("hi", NONSENSE, INJECTED "it's me Jim" WHAT THE F$)
write join("hi",,, "Jim")

write join(join("hi" , (((( "Jim") )

write join("NEVER") IGNORED join("GONNA")
write join() IGNORED join("GONNA")
write join() LET

write join("YOU", join("JOIN"))
"##;
    let expected = r##"nonsense = int(input())
print("hi it's me Jim")
print("hi " + str(nonsense) + " it's me Jim")
print("join("hi",,, "Jim")")
print("hi Jim")
print("NEVER")
print("GONNA")
print("join() LET")
print("YOU JOIN")
"##;

    test_stub_builder(generator, expected);
}

#[test]
fn test_stub_statement() {
    let generator = r##"STATEMENT
There can only be one per stub

STATEMENT this gets ignored
This should start here
STATEMENT
override the previous statement
     and end here (no spaces both sides)   


read n:int
loop n loop n loop n write crazy
right?
"##;
    let expected = r##"# This should start here
# STATEMENT
# override the previous statement
# and end here (no spaces both sides)

n = int(input())
for i in range(n):
    for j in range(n):
        for k in range(n):
            print("crazy")
            print("right?")
"##;

    test_stub_builder(generator, expected);
}

#[test]
fn test_stub_output() {
    let generator = r##"write answer0

write answer1

OUTPUT
This goes to answer 0 and 1
OUTPUT
Together with this

OUTPUT
THis goes to Narnia

write answer2

OUTPUT       oops!
This goes to answer 2 but not 3

write answer3
STATEMENT     
baited,     care spaces

STATEMENT
Hello world!
"##;
    let expected = r##"# Hello world!

# This goes to answer 0 and 1
# OUTPUT
# Together with this
print("answer0")
# This goes to answer 0 and 1
# OUTPUT
# Together with this
print("answer1")
# This goes to answer 2 but not 3
print("answer2")
print("answer3")
print("STATEMENT")
print("baited,     care spaces")
"##;

    test_stub_builder(generator, expected);
}

#[test]
fn test_stub_input_1() {
    let generator = r##"read init:int
read x:int

INPUT   ignored
x: some variable
y: some other inexistent variable

read x:int

INPUT
y: some variable that now exists but gets ignored due to previous INPUT

read two:word(50) words:word(50)
read i:int f:float

INPUT
init: the first variable
words:         some words

INPUT
two:a number, duh
i: int
f: float
"##;
    let expected = r##"init = int(input())  # the first variable
x = int(input())  # some variable
x = int(input())
# two: a number, duh
# words: some words
two, words = input().split()
inputs = input().split()
i = int(inputs[0])  # int
f = float(inputs[1])  # float
"##;

    test_stub_builder(generator, expected);
}

#[test]
fn test_stub_input_2() {
    let generator = r##"read A:int

INPUT case sensitive???
a : NEIN NEIN
A : ????
"##;

    let expected = r##"a = int(input())  # ????
"##;

    test_stub_builder(generator, expected);
}

#[test]
fn test_stub_everything() {
    let generator = r##"write many  spaces   here

read L:string(20)

OUTPUT
The spacemaster

read a:word(50) b:word(50)
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

STATEMENT
Head, shoulders knees and toes
Knees and toes

read xCount:int
loopline xCount x:int
loopline xCount y:int z:word(50)
"##;
    let expected = r##"# Head, shoulders knees and toes
# Knees and toes

# The spacemaster
print("many  spaces   here")
l = input()
a, b = input().split()
a_bc = input()  # The alphabet
row = input()  # Your boat
for i in range(n):
    ext, mt = input().split()
for i in range(n):
    inputs = input().split()
    count = int(inputs[0])
    name = inputs[1]
for i in range(q):
    fname = input()
for i in range(4):
    number = int(input())
for i in range(4):
    print("0 0")
x_count = int(input())
for i in input().split():
    x = int(i)
inputs = input().split()
for i in range(x_count):
    y = int(inputs[2*i])
    z = inputs[2*i+1]
"##;

    test_stub_builder(generator, expected);
}

#[test]
fn test_stub_loops_spaces_and_newlines() {
    let generator = r##"read n:int
loop  
  n    
    

  loop 4 
write thing

loop n    
  loop 4      
  loopline
n x:int"##;
    let expected = r##"n = int(input())
for i in range(n):
    for j in range(4):
        print("thing")
for i in range(n):
    for j in range(4):
        for k in input().split():
            x = int(k)
"##;
    test_stub_builder(generator, expected);
}

#[test]
fn test_stub_variable_length() {
    let generator = r##"read n:int
read k:string(n)
loop n read a:string(5)
write answer

INPUT
k: this string is n-sized (irrelevant in python but be wary!)
"##;
    let expected = r##"n = int(input())
k = input()  # this string is n-sized (irrelevant in python but be wary!)
for i in range(n):
    a = input()
print("answer")
"##;

    test_stub_builder(generator, expected);
}

#[test]
fn test_stub_summary() {
    let generator = r##"read anInt:int
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
"##;
    let expected = r##"# This is the statement

an_int = int(input())  # An input comment over anInt
a_float = float(input())
long = int(input())
a_word = input()
boolean = input() != "0"
abc1abc_1a_bc1ab_c1abc_1 = int(input())
string = input()
inputs = input().split()
an_int_2 = int(inputs[0])
a_float_2 = float(inputs[1])
long_2 = int(inputs[2])
a_word_2 = inputs[3]
boolean_2 = inputs[4] != "0"
for i in range(an_int):
    x = int(input())
for i in range(an_int):
    inputs = input().split()
    x = int(inputs[0])
    f = float(inputs[1])
for i in range(an_int):
    for j in range(an_int):
        x, y = [int(k) for k in input().split()]
for i in input().split():
    x = int(i)
for w in input().split():
    pass
inputs = input().split()
for i in range(an_int):
    x = int(inputs[3*i])
    f = float(inputs[3*i+1])
    w = inputs[3*i+2]
# An output comment
print("result")
print(str(an_int) + " " + str(a_float) + " " + str(long) + " " + str(boolean))
print(a_word + " literal " + string)
"##;

    test_stub_builder(generator, expected);
}
