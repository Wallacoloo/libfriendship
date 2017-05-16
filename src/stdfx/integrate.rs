use routing::{adjlist, NodeHandle, Edge, EdgeWeight, EffectId, EffectDesc, EffectMeta, EffectInput, EffectOutput};
use routing::AdjList;
use util::pack_f32;

use super::{delay, f32constant, passthrough};

/// Get the EffectDesc for the integrate effect.
/// Integrate is constructed such that at any given time,
/// y[t] = \sum_{n=0}^t x[n],
/// where x is the input to slot 0,
/// and y is the output from slot 0.
/// 
/// In particular, this Integrate effect is implemented in a binary tree fashion
/// so, for example,
/// y[7] = {(x[0] + x[1]) + (x[2] + x[3])} + 
///        {(x[4] + x[5]) + (x[6] + x[7])}
/// This is done as an attempt to minimize rounding errors by ensuring each
/// addition operand is approximately the same magnitude given a regular input.
pub fn get_desc(bits: u8) -> EffectDesc {
    assert!(bits >= 1); // Minimum size is length=2
    let length = 1 << (bits as u64);
    let half_length = length >> 1;
    let subnode_meta = if bits == 1 {
        passthrough::get_id()
    } else {
        get_id(bits-1)
    };

    let delay_hnd = NodeHandle::new_node_toplevel(1);
    let delayamt_hnd = NodeHandle::new_node_toplevel(2);
    let sub1_hnd = NodeHandle::new_node_toplevel(3);
    let sub2_hnd = NodeHandle::new_node_toplevel(4);

    let delay_data = adjlist::NodeData::Effect(delay::get_id());
    let delayamt_data = adjlist::NodeData::Effect(f32constant::get_id());
    let sub1_data = adjlist::NodeData::Effect(subnode_meta);
    let sub2_data = sub1_data.clone();
    
    // NOTE: half_length guaranteed to fit in f32 because it's a power of two in the range of f32.
    let edge_delayamt = Edge::new(delayamt_hnd, delay_hnd, EdgeWeight::new(pack_f32(half_length as f32), 1)).unwrap();
    let edge_delay_to_sub = Edge::new(delay_hnd, sub1_hnd, EdgeWeight::new(0, 0)).unwrap();
    // Input to delay -> sub1
    let edge_in1 = Edge::new_from_null(delay_hnd, EdgeWeight::new(0, 0));
    // Input to sub2
    let edge_in2 = Edge::new_from_null(sub2_hnd, EdgeWeight::new(0, 0));
    // Output from sub1
    let edge_out1 = Edge::new_to_null(sub1_hnd, EdgeWeight::new(0, 0));
    // Output from sub2
    let edge_out2 = Edge::new_to_null(sub2_hnd, EdgeWeight::new(0, 0));

    let nodes = [(delay_hnd, delay_data), (delayamt_hnd, delayamt_data),
        (sub1_hnd, sub1_data), (sub2_hnd, sub2_data)];
    let edges = [edge_delayamt, edge_delay_to_sub, edge_in1, edge_in2, edge_out1, edge_out2];

    let list = AdjList {
        nodes: nodes.iter().cloned().collect(),
        edges: edges.iter().cloned().collect(),
    };
    let my_name = format!("Integrate{}", length);
    EffectDesc::new(EffectMeta::new(my_name, None,
        collect_arr!{[ (0, EffectInput::new("source".into(), 0)) ]},
        collect_arr!{[ (0, EffectOutput::new("result".into(), 0)) ]},
    ), list)
}

pub fn get_id(bits: u8) -> EffectId {
    get_desc(bits).id()
}
