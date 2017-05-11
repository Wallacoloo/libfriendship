use routing::{NodeHandle, Edge, EdgeWeight, EffectMeta, EffectDesc};
use routing::AdjList;

/// Get the EffectDesc for the Passthrough effect.
/// The passthrough effect takes all data input to slot 0 and sends it to
/// output slot 0.
pub fn get_desc() -> EffectDesc {
    let edge = Edge::new_from_null(NodeHandle::toplevel(), EdgeWeight::new(0, 0, 0, 0));
    let nodes = [];
    let edges = [edge];

    let list = AdjList {
        nodes: nodes.iter().cloned().collect(),
        edges: edges.iter().cloned().collect(),
    };
    let meta = get_meta();
    EffectDesc::new(meta, list)
}

pub fn get_meta() -> EffectMeta {
    EffectMeta::new("Passthrough".into(), None, None)
}
