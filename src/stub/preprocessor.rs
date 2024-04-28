use super::{renderer::Renderer, Stub};

pub trait RenderableCmd: Renderable + std::fmt::Debug + DynClone {}

pub trait Renderable {
    fn render(&self, renderer: &Renderer) -> String;
}

pub trait Preprocessor {
    fn transform(stub: &mut Stub);
}

// Workaround for trait object to implement Clone:
// https://users.rust-lang.org/t/how-to-deal-with-the-trait-cannot-be-made-into-an-object-error-in-rust-which-traits-are-object-safe-and-which-aint/90620/3
// Playground:
// https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=ba1fd8301dddf385f6f9f56b8316a4df

pub trait DynClone {
    fn dyn_clone(&self) -> Box<dyn RenderableCmd>;
}

impl<T: RenderableCmd + Clone + 'static> DynClone for T {
    fn dyn_clone(&self) -> Box<dyn RenderableCmd> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn RenderableCmd> {
    fn clone(&self) -> Self {
        (**self).dyn_clone()
    }
}
