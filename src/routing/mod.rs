/// Contains all functionality necessary to define the mathematical relationships that govern
/// each sample's value.
/// It's the Renderer's job to determine the most optimial order to computations to satisfy these
/// mathematical relationships.

mod effect;
mod routegraph;
mod sinusoid;

// re-export the things we want public
pub use self::routegraph::{LeafNode, PwLine, RouteEdge, RouteNode, RouteGraph, Sinusoid, WeakNodeHandle};
