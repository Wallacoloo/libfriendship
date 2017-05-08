use url::Url;

use routing::{adjlist, NodeHandle, DagHandle, Edge, EdgeWeight, EffectMeta, EffectDesc};
use routing::AdjList;
use util::pack_f32;

/// Get the EffectDesc for the convolve effect.
/// Convolve is constructed such that at any given time,
/// y[t] = \sum_{n=0}^{LEN-1} x[t-n] * SLOT_{n+1}[t]
/// 
/// In other words, the convolution coefficients are insert to SLOT_1 through
/// SLOT_{LEN} and the audio to be convolved is input to SLOT_0.
/// LEN is the length of the convolution kernel.
pub fn get_desc(bits: u8) -> EffectDesc {
    // maximum length = 2^32-2 because of slot numbering
    assert!(bits < 32 && bits != 0);
    let half_length = 1 << ((bits-1) as u64);
    let subnode_name = if bits == 1 {
        "Multiply".to_string()
    } else {
        format!("Integrate{}", half_length)
    };
    let my_name = format!("Convolve{}", 2*half_length);

    let delay_hnd = NodeHandle::new_node(DagHandle::toplevel(), 1);
    let delayamt_hnd = NodeHandle::new_node(DagHandle::toplevel(), 2);
    let sub1_hnd = NodeHandle::new_node(DagHandle::toplevel(), 3);
    let sub2_hnd = NodeHandle::new_node(DagHandle::toplevel(), 4);

    let delay_data = adjlist::NodeData::Effect(
        EffectMeta::new("Delay".to_string(), None, [Url::parse("primitive:///Delay").unwrap()].iter().cloned())
    );
    let delayamt_data = adjlist::NodeData::Effect(
        EffectMeta::new("F32Constant".to_string(), None, [Url::parse("primitive:///F32Constant").unwrap()].iter().cloned())
    );
    let sub1_data = adjlist::NodeData::Effect(
        EffectMeta::new(subnode_name, None, Vec::new().into_iter())
    );
    let sub2_data = sub1_data.clone();
    
    // NOTE: half_length guaranteed to fit in f32 because it's a power of two in the range of f32.
    let edge_delayamt = Edge::new(delayamt_hnd, delay_hnd, EdgeWeight::new(pack_f32(half_length as f32), 0, 1, 0)).unwrap();
    let edge_delay_to_sub = Edge::new(delay_hnd, sub2_hnd, EdgeWeight::new(0, 0, 0, 0)).unwrap();
    // Input to sub1
    let edge_in1 = Edge::new_from_null(sub1_hnd, EdgeWeight::new(0, 0, 0, 0));
    // Input to delay -> sub2
    let edge_in2 = Edge::new_from_null(delay_hnd, EdgeWeight::new(0, 0, 0, 0));
    // Output from sub1
    let edge_out1 = Edge::new_to_null(sub1_hnd, EdgeWeight::new(0, 0, 0, 0));
    // Output from sub2
    let edge_out2 = Edge::new_to_null(sub2_hnd, EdgeWeight::new(0, 0, 0, 0));

    // Lower half of kernel parameters
    let edges_to_sub1 = (0..half_length).map(|i| {
        Edge::new_from_null(sub1_hnd, EdgeWeight::new(1+i, 0, 1+i, 0))
    });
    let edges_to_sub2 = (0..half_length).map(|i| {
        Edge::new_from_null(sub2_hnd, EdgeWeight::new(1+half_length+i, 0, 1+i, 0))
    });

    let edges = [edge_delayamt, edge_delay_to_sub, edge_in1, edge_in2, edge_out1, edge_out2].iter().cloned()
        .chain(edges_to_sub1)
        .chain(edges_to_sub2)
        .collect();
    let nodes = [(delay_hnd, delay_data), (delayamt_hnd, delayamt_data),
        (sub1_hnd, sub1_data), (sub2_hnd, sub2_data)].iter().cloned().collect();

    let list = AdjList { nodes, edges };
    let meta = EffectMeta::new(my_name, None, Vec::new().into_iter());
    EffectDesc::new(meta, list)
}
