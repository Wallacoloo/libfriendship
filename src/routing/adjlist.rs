/// `RouteGraph`s cannot be serialized/deserialized natively.
/// Instead, we allow an `AdjList` type that contains the same data
/// but is static.


use super::routegraph::{NodeHandle, Edge};
use super::effect::EffectId;

#[derive(Serialize, Deserialize)]
pub struct AdjList {
    pub nodes: Vec<(NodeHandle, EffectId)>,
    pub edges: Vec<Edge>,
}

