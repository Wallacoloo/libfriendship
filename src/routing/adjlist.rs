/// RouteGraphs cannot be serialized/deserialized natively.
/// Instead, we implement a to_adjlist and from_adjlist function, and serialize
/// adjacency lists.


use super::routegraph::{NodeHandle, DagHandle, Edge};
use super::effect::EffectId;

#[derive(Clone)]
#[derive(Serialize, Deserialize)]
pub enum NodeData {
    Effect(EffectId),
    Graph(DagHandle),
}

#[derive(Serialize, Deserialize)]
pub struct AdjList {
    pub nodes: Vec<(NodeHandle, NodeData)>,
    pub edges: Vec<Edge>,
}

