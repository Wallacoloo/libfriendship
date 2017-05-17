use std;

use routing::{adjlist, NodeHandle, Edge, EdgeWeight, EffectId, EffectDesc, EffectMeta, EffectOutput};
use routing::AdjList;
use util::pack_f32;

use super::f32constant;

/// Get the EffectDesc for a Hamming window of size N.
/// Each output is simply the weights for the corresponding index into the
/// window.
/// The coefficients used are the optimal Hamming coeffs,
/// alpha = 0.53836, beta = 0.46164.
pub fn get_desc(n: u32) -> EffectDesc {
    assert!(n > 1);
    const TWO_PI: f64 = std::f64::consts::PI * 2.0f64;
    let alpha = 0.53836f64;
    let beta = 0.46164f64;
    let len_1 = (n - 1) as f64;
    let weights = (0..n).map(|i| {
        alpha - beta * (TWO_PI*i as f64)/len_1
    });

    let handles = || (0..n).map(|i| {
        NodeHandle::new_node_toplevel(1+i)
    });
    let node_data = (0..n).map(|_| {
        adjlist::NodeData::Effect(f32constant::get_id())
    });
    let edges = weights.zip(handles()).enumerate().map(|(i, (weight, hnd))| {
        Edge::new_to_null(hnd, EdgeWeight::new(pack_f32(weight as f32), i as u32))
    }).collect();
    let nodes = handles().zip(node_data).collect();

    let outputs = (0..n).map(|i| {
        EffectOutput::new(format!("weight[{}]", 1), 0)
    });

    let list = AdjList { nodes, edges };
    let my_name = format!("Hamming{}", n);
    EffectDesc::new(EffectMeta::new(my_name, None,
        collect_arr!{[]},
        outputs.collect(),
    ), list)
}

pub fn get_id(n: u32) -> EffectId {
    get_desc(n).id()
}
