/// Contains all functionality necessary to define the mathematical relationships that govern
/// each sample's value.
/// It's the Renderer's job to determine the most optimial order to computations to satisfy these
/// mathematical relationships.

mod adjlist;
mod effect;
mod graphwatcher;
mod routegraph;
//mod sinusoid;

// re-export the things we want public
pub use self::effect::EffectMeta;
pub use self::graphwatcher::GraphWatcher;
pub use self::routegraph::{Edge, NodeData, NodeHandle};
