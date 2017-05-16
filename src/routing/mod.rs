/// Contains all functionality necessary to define the mathematical relationships that govern
/// each sample's value.
/// It's the Renderer's job to determine the most optimial order to computations to satisfy these
/// mathematical relationships.

pub mod adjlist;
pub mod effect;
mod graphwatcher;
pub mod routegraph;
mod nullable_int;

// re-export the things we want public
pub use self::effect::{Effect, EffectDesc, EffectId, EffectMeta};
pub use self::graphwatcher::GraphWatcher;
pub use self::routegraph::{DagHandle, Edge, EdgeWeight, NodeData, NodeHandle, RouteGraph};
pub use self::adjlist::AdjList;
