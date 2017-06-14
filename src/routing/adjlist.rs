/// `RouteGraph`s cannot be serialized/deserialized natively.
/// Instead, we allow an `AdjList` type that contains the same data
/// but is static.


use super::routegraph::{NodeHandle, Edge};
use super::effect::EffectId;

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct AdjList {
    // TODO: No need to store NodeHandles; just assume they start counting from 1.
    pub nodes: Vec<(NodeHandle, EffectId)>,
    pub edges: Vec<Edge>,
}

