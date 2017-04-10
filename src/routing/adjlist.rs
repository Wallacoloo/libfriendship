/// RouteGraphs cannot be serialized/deserialized natively.
/// Instead, we implement a to_adjlist and from_adjlist function, and serialize
/// adjacency lists.

use std::collections::hash_map::HashMap;
use std::collections::hash_set::HashSet;

use super::routegraph::{NodeHandle, DagHandle, Edge};
use super::effect::EffectDesc;

#[derive(Serialize, Deserialize)]
pub enum NodeData {
    Effect(EffectDesc),
    Graph(DagHandle),
}

#[derive(Serialize, Deserialize)]
pub struct AdjList {
    pub nodes: HashMap<NodeHandle, NodeData>,
    pub edges: HashSet<Edge>,
}

