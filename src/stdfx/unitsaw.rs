use routing::{adjlist, NodeHandle, Edge, EdgeWeight, EffectId, EffectDesc, EffectMeta};
use routing::AdjList;
use util::pack_f32;

use super::{f32constant, modulo_one, multiply};

/// Get the EffectDesc for a function that computes a sawtooth wave, given
/// an index. Mathematically,
/// y = -1 + 2*(x mod 1),
/// where x is the index (slot 0 input) and y is the sawtooth (slot 0 output)
pub fn get_desc() -> EffectDesc {
    let const_hnd = NodeHandle::new_node_toplevel(1);
    let mod_hnd = NodeHandle::new_node_toplevel(2);
    let mult_hnd = NodeHandle::new_node_toplevel(3);

    let const_data = adjlist::NodeData::Effect(f32constant::get_id());
    let mod_data = adjlist::NodeData::Effect(modulo_one::get_id());
    let mult_data = adjlist::NodeData::Effect(multiply::get_id());

    // x mod 1
    let edge_in = Edge::new_from_null(mod_hnd, EdgeWeight::new(0, 0, 0, 0));
    // 2*[x mod 1]
    let edge_double = Edge::new(mod_hnd, mult_hnd, EdgeWeight::new(0, 0, 0, 0)).unwrap();
    let edge_double_const = Edge::new(const_hnd, mult_hnd, EdgeWeight::new(pack_f32(2.0f32), 0, 1, 0)).unwrap();
    // [2*(x mod 1)] -> output
    let edge_mul_out = Edge::new_to_null(mult_hnd, EdgeWeight::new(0, 0, 0, 0));
    // -1 -> output
    let edge_const_out = Edge::new_to_null(const_hnd, EdgeWeight::new(pack_f32(-1.0f32), 0, 0, 0));
    
    let nodes = [(const_hnd, const_data), (mod_hnd, mod_data), (mult_hnd, mult_data)];
    let edges = [edge_in, edge_double, edge_double_const, edge_mul_out, edge_const_out];
    let list = AdjList {
        nodes: nodes.iter().cloned().collect(),
        edges: edges.iter().cloned().collect(),
    };
    EffectDesc::new(EffectMeta::new("UnitSaw".into(), None), list)
}

pub fn get_id() -> EffectId {
    get_desc().id()
}
