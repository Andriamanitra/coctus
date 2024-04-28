use super::Renderable;
use crate::stub::{Cmd, Stub};

pub fn transform(stub: &mut Stub) {
    let old_commands = stub.commands.drain(..).peekable();

    // while let Some(cmd) = old_commands.next() {
    //     
    // }

    let (mut cmds, mut leftover) = old_commands.rev().fold((vec![], vec![]), |(mut cmds, mut reads), cmd| {
        if matches!(cmd, Cmd::Read(_)) {
            reads.push(cmd)
        } else {
            if !reads.is_empty() {
                let read_batch = ReadBatch {
                    line_readers: reads.drain(..).rev().collect(),
                    nested_cmds: cmds.drain(..).rev().collect(),
                };

                cmds.push(Cmd::External(Box::new(read_batch)));
            }

            cmds.push(cmd);
        }

        (cmds, reads)
    });

    if !leftover.is_empty() {
        let read_batch = ReadBatch {
            line_readers: leftover.drain(..).rev().collect(),
            nested_cmds: cmds.drain(..).rev().collect(),
        };

        cmds.push(Cmd::External(Box::new(read_batch)));
    }

    stub.commands = cmds;
    stub.commands.reverse();
}

#[derive(Debug, Clone)]
struct ReadBatch {
    pub line_readers: Vec<Cmd>,
    pub nested_cmds: Vec<Cmd>,
}

impl Renderable for ReadBatch {
    fn render(&self, renderer: &crate::stub::renderer::Renderer) -> String {
        let nested_string: String =
            self.nested_cmds.iter().map(|cmd| renderer.render_command(cmd, 0)).collect();
        let nested_lines: Vec<&str> = nested_string.lines().collect();

        let read_lines: String =
            self.line_readers.iter().map(|cmd| renderer.render_command(cmd, 0)).collect();
        let read_lines: Vec<&str> = read_lines.lines().collect();

        let mut context = tera::Context::new();
        context.insert("read_lines", &read_lines);
        context.insert("nested_lines", &nested_lines);
        renderer.tera_render("read_batch", &mut context)
    }
}
