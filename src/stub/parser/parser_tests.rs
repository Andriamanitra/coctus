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
    let Cmd::Write { text, .. } = parser.parse_write() else { panic!() };
    assert_eq!(text, "hello world");
}

#[test]
fn parse_write_captures_lines_of_text() {
    let mut parser = Parser::new("hello\nworld");
    let Cmd::Write { text, .. } = parser.parse_write() else { panic!() };
    assert_eq!(text, "hello\nworld");
}

#[test]
fn parse_write_captures_lines_of_text_until_empty_line() {
    let mut parser = Parser::new("hello\nworld\n\nread");
    let Cmd::Write { text, .. } = parser.parse_write() else { panic!() };
    assert_eq!(text, "hello\nworld");
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
    let Cmd::Write { text, .. } = parser.parse_write() else { panic!() };
    assert_eq!(text, "hello join() world");
}

#[test]
fn parse_write_captures_incomplete_write_joins() {
    let mut parser = Parser::new("hello join( world");
    let Cmd::Write { text, .. } = parser.parse_write() else { panic!() };
    assert_eq!(text, "hello join( world");
}

#[test]
fn parse_write_captures_invalid_write_joins() {
    let mut parser = Parser::new("hello join(\"thing\",,) world");
    let Cmd::Write { text, .. } = parser.parse_write() else { panic!() };
    assert_eq!(text, "hello join(\"thing\",,) world");
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
    let Cmd::Write { text, .. } = *inner_cmd else { panic!() };
    assert_eq!(text, "hello world")
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

}
