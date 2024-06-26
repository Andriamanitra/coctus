use super::Renderable;
use crate::stub::renderer::ALPHABET;
use crate::stub::{Cmd, Stub, VarType, VariableCommand};

/// Change the Stub structure into: [ReadDeclarations, MainContents(old_cmds)]
/// This is relevant for Pascal.
#[derive(Debug, Clone)]
struct MainWrapper {
    // Read declarations that should go on top of the main function.
    // render declaration: int c;
    // render read (usual): int c;\nscanf("%d", c);
    pub forward_declarations: Vec<VariableCommand>,
    // The main function contents.
    pub main_content: Vec<Cmd>,
}

/// Edit stub to allow for Pascal-style forward declarations.
///
/// Wraps all of the commands in a stub that contains:
/// - Forward declarations
/// - The rest of the code
///
/// Traverses through the stub commands, taking all declared variables.
/// Leaves only one MainWrapper command in the stub.
/// Introduces two new templates:
/// - `forward_declaration` - similar to a read_one declares one single
///   variable, includes all the fields in a VariableCommand (not nested under
///   `var`)
/// - `main_wrapper` - wraps all of the code, contains `forward_declarations`
///   (the above resource, rendered) and `main_content`
pub fn transform(stub: &mut Stub) {
    let mut max_nested_depth = 0;

    let mut forward_declarations: Vec<VariableCommand> = stub
        .commands
        .iter()
        .filter_map(|cmd| {
            let (cmd, nested_depth) = unpack_cmd(cmd, 0);

            if nested_depth > max_nested_depth {
                max_nested_depth = nested_depth;
            }

            match cmd {
                Cmd::LoopLine {
                    variables: var_cmds, ..
                } => Some(var_cmds.into_iter()),
                Cmd::Read(var_cmds) => Some(var_cmds.into_iter()),
                _ => None,
            }
        })
        .flatten()
        .collect();

    // Add the loop indices to the declarations
    forward_declarations.extend(ALPHABET[0..max_nested_depth].iter().map(|loop_var| VariableCommand {
        ident: loop_var.to_string(),
        var_type: VarType::Int,
        max_length: None,
        input_comment: String::new(),
    }));

    let mut seen = std::collections::BTreeSet::<String>::new();
    forward_declarations.retain(|var_cmd| seen.insert(var_cmd.ident.clone()));

    let wrapper = MainWrapper {
        forward_declarations,
        main_content: stub.commands.drain(..).collect(),
    };

    stub.commands = vec![Cmd::External(Box::new(wrapper))];
}

fn unpack_cmd(cmd: &Cmd, nested_depth: usize) -> (Cmd, usize) {
    match cmd {
        Cmd::Loop {
            count_var: _,
            command: subcmd,
        } => unpack_cmd(subcmd, nested_depth + 1),
        Cmd::LoopLine { .. } => (cmd.clone(), nested_depth + 1),
        _ => (cmd.clone(), nested_depth),
    }
}

impl Renderable for MainWrapper {
    fn render(&self, renderer: &crate::stub::renderer::Renderer) -> String {
        let main_contents_str: String =
            self.main_content.iter().map(|cmd| renderer.render_command(cmd, 0)).collect();
        let main_contents: Vec<&str> = main_contents_str.lines().collect();

        let forward_declarations: Vec<String> =
            self.forward_declarations.iter().map(|vc| vc.render(renderer)).collect();

        let mut context = tera::Context::new();
        context.insert("forward_declarations", &forward_declarations);
        context.insert("main_contents", &main_contents);
        renderer.tera_render("main_wrapper", &mut context)
    }
}

impl Renderable for VariableCommand {
    fn render(&self, renderer: &crate::stub::renderer::Renderer) -> String {
        let mut context =
            tera::Context::from_serialize(self).expect("VariableCommand should be serializable");
        context.insert("ident", &renderer.lang.variable_name_options.transform_variable_name(&self.ident));
        renderer.tera_render("forward_declarations", &mut context).trim().to_string()
    }
}
