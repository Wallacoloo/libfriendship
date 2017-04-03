/// Contains all functionality necessary to define the mathematical relationships that govern
/// each sample's value.
/// It's the Renderer's job to determine the most optimial order to computations to satisfy these
/// mathematical relationships.

mod effect;
mod graphwatcher;
mod routegraph;
//mod sinusoid;

// re-export the things we want public
pub use self::effect::EffectDesc;
pub use self::routegraph::{Edge, NodeHandle};
