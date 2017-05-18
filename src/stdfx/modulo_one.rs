use routing::{adjlist, NodeHandle, Edge, EdgeWeight, EffectId, EffectDesc, EffectMeta, EffectInput, EffectOutput};
use routing::AdjList;
use util::pack_f32;

use super::{f32constant, modulo};

/// Get the `EffectDesc` for an effect that calculates y = (x mod 1.0),
/// where x is input into slot 0, y is output from slot 0.
pub fn get_desc() -> EffectDesc {
    let const_hnd = NodeHandle::new_node_toplevel(1);
    let mod_hnd = NodeHandle::new_node_toplevel(2);

    let const_data = f32constant::get_id();
    let mod_data = modulo::get_id();

    let edge_in = Edge::new_from_null(mod_hnd, EdgeWeight::new(0, 0));
    let edge_out = Edge::new_to_null(mod_hnd, EdgeWeight::new(0, 0));
    // edge to tell Modulo to modulo by 1.0.
    let edge_const = Edge::new(const_hnd, mod_hnd, EdgeWeight::new(pack_f32(1.0f32), 1)).unwrap();

    let nodes = collect_arr!{[(const_hnd, const_data), (mod_hnd, mod_data)]};
    let edges = collect_arr!{[edge_in, edge_out, edge_const]};
    let list = AdjList{ nodes, edges };

    EffectDesc::new(EffectMeta::new("ModuloOne".into(), None,
        collect_arr!{[ EffectInput::new("source".into(), 0) ]},
        collect_arr!{[ EffectOutput::new("result".into(), 0) ]},
    ), list)
}

pub fn get_id() -> EffectId {
    get_desc().id()
}
