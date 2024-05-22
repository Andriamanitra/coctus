pub mod forward_declarations;
pub mod lisp_like;

use dyn_clone::DynClone;

use super::renderer::Renderer;
use super::Stub;

/// A function that transforms a stub.
///
/// The intended use case of these preprocessor functions is to rearrange Stub
/// in order to add functionality through custom Renderable types.
pub type Preprocessor = fn(&mut Stub) -> ();

/// A type that provides custom rendering logic.
///
/// This is the primary method of extending the stub rendering system of coctus
/// without affecting existing templates. Any type that implements this trait
/// may be wrapped in Cmd::External and its render method will be called by
/// Renderer.
pub trait Renderable: std::fmt::Debug + DynClone {
    fn render(&self, renderer: &Renderer) -> String;
}

dyn_clone::clone_trait_object!(Renderable);
