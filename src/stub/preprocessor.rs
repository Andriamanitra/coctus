pub mod s_expressions;

use dyn_clone::DynClone;

use super::renderer::Renderer;
use super::Stub;

pub type Preprocessor = fn(&mut Stub) -> ();

pub trait Renderable: std::fmt::Debug + DynClone {
    fn render(&self, renderer: &Renderer) -> String;
}

dyn_clone::clone_trait_object!(Renderable);
