use url::Url;

use routing::{adjlist, NodeHandle, DagHandle, Edge, EdgeWeight, EffectMeta, EffectDesc};
use routing::AdjList;

/// Get the EffectDesc for the convolve effect.
/// Convolve is constructed such that at any given time,
/// y[t] = \sum_{n=0}^{LEN-1} x[t-n] * SLOT_{n+1}[t]
/// 
/// In other words, the convolution coefficients are insert to SLOT_1 through
/// SLOT_{LEN} and the audio to be convolved is input to SLOT_0.
/// LEN is the length of the convolution kernel.
pub fn get_desc(len: u32) -> EffectDesc {
    // TODO: This implementation is limited to small sizes due to the use of
    // DelayArray & internal node limits.
    let my_name = format!("Convolve{}", len);

    let delay_hnd = NodeHandle::new_node(DagHandle::toplevel(), 1);
    let mult_hnds: Vec<_> = (0..len).map(|i| {
        NodeHandle::new_node(DagHandle::toplevel(), i+2)
    }).collect();

    let delay_data = adjlist::NodeData::Effect(
        EffectMeta::new("DelayArray".to_string(), None, Vec::new().into_iter())
    );
    let mult_datas = (0..len).map(|_| {
        adjlist::NodeData::Effect(
            EffectMeta::new("Multiply".to_string(), None, [Url::parse("primitive:///Multiply").unwrap()].iter().cloned())
        )
    });

    // edge from autio input to delayer
    let audio_in_edge = Edge::new_from_null(delay_hnd, EdgeWeight::new(0, 0, 0, 0));
    // edges from kernel inputs to the multipliers
    let kernel_edges: Vec<_> = (0..len).map(|i| {
        Edge::new_from_null(mult_hnds[i as usize].clone(), EdgeWeight::new(i+1, 0, 1, 0))
    }).collect();
    // edges from each delayed output to their multiplier
    let delay_to_mult_edges: Vec<_> = (0..len).map(|i| {
        Edge::new(delay_hnd, mult_hnds[i as usize].clone(), EdgeWeight::new(i, 0, 0, 0)).unwrap()
    }).collect();
    // edges from the multipliers to the output (summed)
    let out_edges: Vec<_> = (0..len).map(|i| {
        Edge::new_to_null(mult_hnds[i as usize].clone(), EdgeWeight::new(0, 0, 0, 0))
    }).collect();

    let edges = vec![audio_in_edge].into_iter()
        .chain(kernel_edges)
        .chain(delay_to_mult_edges)
        .chain(out_edges)
        .collect();
    let nodes = vec![(delay_hnd, delay_data)].into_iter().chain(
        mult_hnds.into_iter().zip(mult_datas)
    ).collect();

    let list = AdjList { nodes, edges };
    let meta = EffectMeta::new(my_name, None, Vec::new().into_iter());
    EffectDesc::new(meta, list)
}
