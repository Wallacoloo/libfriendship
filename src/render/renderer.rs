/// Declares traits that allow for rendering a Tree.
use routing::RouteTree;

pub trait Renderer {
    fn step(&self, tree: &RouteTree) -> &[f32];
}
