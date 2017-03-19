/// Declares traits that allow for rendering an EffectGraph
use routing::effect::{EffectEdge};
use routing::RouteGraph;

pub trait Renderer {
    fn step(&mut self, tree: &EffectGraph, output: &mut [f32]);
    /// Called whenever a new edge is added to the EffectGraph of interest
    fn edge_added(&mut self, edge: EffectEdge);
    /// Called whenever an edge is removed from the EffectGraph of interest
    fn edge_removed(&mut self, edge: EffectEdge);
}
