use super::Renderable;
use crate::stub::{Cmd, Stub};

//
// #[derive(Debug, Clone)]
// struct ReadDeclaration {
//     pub read_dclr: Read,
// }
//

/// Change the Stub structure into: [ReadDeclarations, MainContents(old_cmds)]
/// This is relevant for Pascal.
#[derive(Debug, Clone)]
struct ReadDeclarationsWrapper {
    // Read declarations that should go on top of the main function.
    // TODO: these need to be wrapped again so that the renderer know
    // that it has to only declare them (and not call render::render_read)
    // render declaration: int c;
    // render read (usual): int c;\nscanf("%d", c);
    pub read_declarations: Vec<Cmd>,
    // The main function contents.
    pub main_content: Vec<Cmd>,
}

pub fn transform(stub: &mut Stub) {
    let mut old_commands = stub.commands.drain(..).rev().peekable();

    let mut cmds = Vec::new();
    let mut reads = Vec::new();

    while let Some(cmd) = old_commands.next() {
        // TODO: add reads inside loops
        if matches!(cmd, Cmd::Read(_)) {
            reads.push(cmd.clone())
        }
        cmds.push(cmd);
    }

    // cmds.reverse();
    drop(old_commands);
    let wrapper = ReadDeclarationsWrapper {
        read_declarations: reads.drain(..).rev().collect(),
        main_content: cmds.drain(..).rev().collect(),
    };

    stub.commands = vec![Cmd::External(Box::new(wrapper))];
}

impl Renderable for ReadDeclarationsWrapper {
    fn render(&self, renderer: &crate::stub::renderer::Renderer) -> String {
        let main_contents_str: String =
            self.main_content.iter().map(|cmd| renderer.render_command(cmd, 0)).collect();
        let main_contents: Vec<&str> = main_contents_str.lines().collect();

        let read_declarations_str: String =
            self.read_declarations.iter().map(|cmd| renderer.render_command(cmd, 0)).collect();
        let read_declarations: Vec<&str> = read_declarations_str.lines().collect();

        let mut context = tera::Context::new();
        context.insert("read_declarations", &read_declarations);
        context.insert("main_contents", &main_contents);
        renderer.tera_render("init_read_declarations", &mut context)
    }
}
