/// Declares traits that allow for rendering a Tree.
use routing::RouteTree;

pub trait Renderer {
    fn step(&mut self, tree: &RouteTree, output: &mut [f32]);
}
