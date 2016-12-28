/// Declares traits that allow for rendering a Tree.
use routing::RouteGraph;

pub trait Renderer {
    fn step(&mut self, tree: &RouteGraph, output: &mut [f32]);
}
