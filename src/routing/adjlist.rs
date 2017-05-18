/// `RouteGraph`s cannot be serialized/deserialized natively.
/// Instead, we allow an `AdjList` type that contains the same data
/// but is static.


use super::routegraph::{NodeHandle, DagHandle, Edge};
use super::effect::EffectId;

#[derive(Clone)]
#[derive(Serialize, Deserialize)]
pub enum NodeData {
    Effect(EffectId),
}

#[derive(Serialize, Deserialize)]
pub struct AdjList {
    pub nodes: Vec<(NodeHandle, NodeData)>,
    pub edges: Vec<Edge>,
}

