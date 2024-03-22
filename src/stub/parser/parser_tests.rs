#![cfg_attr(rustfmt, rustfmt_skip)]

use indoc::indoc;

use super::*;

#[test]
fn parse_read_parses_variable_list() {
    let mut parser = Parser::new("a:int b:long");
    let Cmd::Read(variables) = parser.parse_read() else { panic!() };
    assert_eq!(variables.len(), 2)
}

#[test]
#[should_panic]
fn parse_read_panics_without_variables() {
    Parser::new("").parse_read();
}

#[test]
#[should_panic]
fn parse_read_panics_without_variable_type() {
    Parser::new("a").parse_read();
}

#[test]
#[should_panic]
fn parse_read_panics_with_variable_of_unknown_type() {
    Parser::new("a:enum").parse_read();
}

#[test]
#[should_panic]
fn parse_read_panics_with_sized_variable_without_size() {
    Parser::new("a:word").parse_read();
}

#[test]
fn parse_write_captures_text() {
    let mut parser = Parser::new("hello world");
    let Cmd::Write { lines, .. } = parser.parse_write() else { panic!() };
    assert_eq!(lines[0], "hello world");
}

#[test]
fn parse_write_captures_lines_of_text() {
    let mut parser = Parser::new("hello\nworld");
    let Cmd::Write { lines, .. } = parser.parse_write() else { panic!() };
    assert_eq!(lines, vec!["hello", "world"]);
}

#[test]
fn parse_write_captures_lines_of_text_until_empty_line() {
    let mut parser = Parser::new("hello\nworld\n\nread");
    let Cmd::Write { lines, .. } = parser.parse_write() else { panic!() };
    assert_eq!(lines, vec!["hello", "world"]);
}

#[test]
fn parse_write_returns_write_joins() {
    let mut parser = Parser::new("join(\"hello\", world)");
    let Cmd::WriteJoin { join_terms, .. } = parser.parse_write() else { panic!() };

    let [
        JoinTerm { name: first_term,  .. }, 
        JoinTerm { name: second_term, .. }
    ] = join_terms.as_slice() else { panic!() };

    assert_eq!(first_term, "hello");
    assert_eq!(second_term, "world");
}

#[test]
fn parse_write_captures_empty_write_joins() {
    let mut parser = Parser::new("hello join() world");
    let Cmd::Write { lines, .. } = parser.parse_write() else { panic!() };
    assert_eq!(lines[0], "hello join() world");
}

#[test]
fn parse_write_captures_incomplete_write_joins() {
    let mut parser = Parser::new("hello join( world");
    let Cmd::Write { lines, .. } = parser.parse_write() else { panic!() };
    assert_eq!(lines[0], "hello join( world");
}

#[test]
fn parse_write_captures_invalid_write_joins() {
    let mut parser = Parser::new("hello join(\"thing\",,) world");
    let Cmd::Write { lines, .. } = parser.parse_write() else { panic!() };
    assert_eq!(lines[0], "hello join(\"thing\",,) world");
}

#[test]
fn parse_loop_accepts_literal_count() {
    let mut parser = Parser::new("2 read a:int");
    let Cmd::Loop { count_var, .. } = parser.parse_loop() else { panic!() };
    assert_eq!(count_var, "2")
}

#[test]
fn parse_loop_accepts_identifier_count() {
    let mut parser = Parser::new("n read a:int");
    let Cmd::Loop { count_var, .. } = parser.parse_loop() else { panic!() };
    assert_eq!(count_var, "n")
}

#[test]
#[should_panic]
fn parse_loop_panics_without_identifier() {
    Parser::new("read a:int").parse_loop();
}

#[test]
#[should_panic]
fn parse_loop_panics_without_command() {
    Parser::new("n").parse_loop();
}

#[test]
#[should_panic]
fn parse_loop_panics_with_unknown_command() {
    Parser::new("n dance").parse_loop();
}

#[test]
fn parse_loop_accepts_read_command() {
    let mut parser = Parser::new("n read a:int b:long c:bool");
    let Cmd::Loop { command: inner_cmd, ..  } = parser.parse_loop() else { panic!() };
    let Cmd::Read(vars) = *inner_cmd else { panic!() };
    assert_eq!(vars.len(), 3)
}

#[test]
fn parse_loop_accepts_write_command() {
    let mut parser = Parser::new("n write hello world");
    let Cmd::Loop { command: inner_cmd, ..  } = parser.parse_loop() else { panic!() };
    let Cmd::Write { lines, .. } = *inner_cmd else { panic!() };
    assert_eq!(lines[0], "hello world")
}

#[test]
fn parse_loop_accepts_loopline() {
    let mut parser = Parser::new("n loopline 3 x:int");
    let Cmd::Loop { command: inner_cmd, ..  } = parser.parse_loop() else { panic!() };
    let Cmd::LoopLine { count_var, variables } = *inner_cmd else { panic!() };
    assert_eq!(count_var, "3");
    assert_eq!(variables.len(), 1);
}

#[test]
fn parse_loop_can_be_nested_infinitely() {
    let stub_text = "n loop ".repeat(20) + "n read a:int";
    let mut parser = Parser::new(stub_text.as_str());
    let mut current_cmd = parser.parse_loop();
    while let Cmd::Loop { command: inner_cmd, count_var  } = current_cmd {
        current_cmd = *inner_cmd;
        assert_eq!(count_var, "n");
    } 
    let Cmd::Read(vars) = current_cmd else { panic!() };
    assert_eq!(vars.len(), 1)
}

#[test]
fn parse_loop_tolerates_newlines_around_count() {
    let mut parser = Parser::new(" \nn \nread x:int");
    let Cmd::Loop { command: inner_cmd, ..  } = parser.parse_loop() else { panic!() };
    let Cmd::Read(vars) = *inner_cmd else { panic!() };
    assert_eq!(vars.len(), 1);
}

#[test]
fn parse_loopline_parses_counter_and_variables() {
    let mut parser = Parser::new("n a:int b:long c:word(50)");
    let Cmd::LoopLine { count_var, variables } = parser.parse_loopline() else { panic!() };
    assert_eq!(count_var, "n");
    assert_eq!(variables.len(), 3);
}

#[test]
#[should_panic]
fn parse_loopline_panics_without_counter() {
    Parser::new("").parse_loopline();
}

#[test]
#[should_panic]
fn parse_loopline_panics_without_variables() {
    Parser::new("n").parse_loopline();
}

#[test]
fn parse_input_comment_attaches_comment_to_read() {
    let mut parser = Parser::new(indoc! {r"
        a:int
        INPUT
        a: a number
    "});

    let mut commands = [parser.parse_read()];
    parser.parse_input_comment(&mut commands);
    let Cmd::Read(ref vars) = commands[0] else { panic!() };
    assert_eq!(vars[0].input_comment, "a number");
}

#[test]
fn parse_input_comment_attaches_comment_to_multiple_vars() {
    let mut parser = Parser::new(indoc! {r"
        a:int b:long
        INPUT
        b: A big number
        a: a number
    "});

    let mut commands = [parser.parse_read()];
    parser.parse_input_comment(&mut commands);
    let Cmd::Read(ref vars) = commands[0] else { panic!() };
    assert_eq!(vars[0].input_comment, "a number");
    assert_eq!(vars[1].input_comment, "A big number");
}

#[test]
fn parse_input_comment_ignores_lines_without_variable() {
    let mut parser = Parser::new(indoc! {r"
        a:int b:long
        INPUT
        A WORTHLESS LINE
        a: a number
    "});

    let mut commands = [parser.parse_read()];
    parser.parse_input_comment(&mut commands);
    let Cmd::Read(ref vars) = commands[0] else { panic!() };
    assert_eq!(vars[0].input_comment, "a number");
}

#[test]
fn parse_input_comment_attaches_comment_to_loopline() {
    let mut parser = Parser::new(indoc! {r"
        1 a:int
        INPUT
        a: a number
    "});

    let mut commands = [parser.parse_loopline()];
    parser.parse_input_comment(&mut commands);
    let Cmd::LoopLine { ref variables, .. } = commands[0] else { panic!() };
    assert_eq!(variables[0].input_comment, "a number");
}

#[test]
fn parse_input_comment_attaches_comment_to_read_inside_loop() {
    let mut parser = Parser::new(indoc! {r"
        1 read a:int
        INPUT
        a: a number
    "});
    let mut commands = [parser.parse_loop()];
    parser.parse_input_comment(&mut commands);
    let Cmd::Loop { ref command, .. } = commands[0] else { panic!() };
    let Cmd::Read(variables) = *command.clone() else { panic!() };
    assert_eq!(variables[0].input_comment, "a number");
}

#[test]
fn parse_input_comment_attaches_comment_to_loopline_inside_loop() {
    let mut parser = Parser::new(indoc! {r"
        1 loopline 1 a:int
        INPUT
        a: a number
    "});

    let mut commands = [parser.parse_loop()];
    parser.parse_input_comment(&mut commands);
    let Cmd::Loop { ref command, .. } = commands[0] else { panic!() };
    let Cmd::LoopLine { ref variables, .. } = *command.clone() else { panic!() };
    assert_eq!(variables[0].input_comment, "a number");
}

#[test]
fn parse_output_comment_adds_comment_to_write() {
    let mut parser = Parser::new(indoc! {r"
        Knock You Out

        the OUTPUT keyword is already consumed
        Mama said
    "});

    let mut commands = [parser.parse_write()];
    parser.parse_output_comment(&mut commands);
    let Cmd::Write { ref lines, ref output_comment } = commands[0] else { panic!() };
    assert_eq!(lines[0], "Knock You Out");
    assert_eq!(output_comment[0], "Mama said");
}

#[test]
fn parse_output_comment_adds_comment_to_multiple_writes() {
    let mut parser = Parser::new(indoc! {r"
        Knock You Out

        Eat your vegetables

        the OUTPUT keyword is already consumed
        Mama said
    "});

    let mut commands = [parser.parse_write(), parser.parse_write()];
    parser.parse_output_comment(&mut commands);

    let Cmd::Write { ref lines, ref output_comment } = commands[0] else { panic!() };
    let Cmd::Write { lines: ref second_lines, output_comment: ref second_comment } = commands[1] else { panic!() };

    assert_eq!(lines[0], "Knock You Out");
    assert_eq!(output_comment[0], "Mama said");

    assert_eq!(second_lines[0], "Eat your vegetables");
    assert_eq!(second_comment[0], "Mama said");
}

#[test]
fn parse_output_comment_does_not_overwrite() {
    let mut parser = Parser::new(indoc! {r"
        Knock You Out

        the OUTPUT keyword is already consumed
        Mama said

        the OUTPUT keyword is already consumed
        Daddy said
    "});

    let mut commands = [parser.parse_write()];
    parser.parse_output_comment(&mut commands);
    parser.parse_output_comment(&mut commands); // Parses "Daddy said" but does not use it

    let Cmd::Write { ref lines, ref output_comment } = commands[0] else { panic!() };

    assert_eq!(lines[0], "Knock You Out");
    assert_eq!(output_comment[0], "Mama said");
}

#[test]
fn parse_output_comment_adds_comment_to_write_join() {
    let mut parser = Parser::new(indoc! {r##"
        join("Knock", "You", "Out")
        the OUTPUT keyword is already consumed
        Mama said
    "##});

    let mut commands = [parser.parse_write()];
    parser.parse_output_comment(&mut commands);
    let Cmd::WriteJoin { ref output_comment, .. } = commands[0] else { panic!() };
    assert_eq!(output_comment[0], "Mama said");
}
