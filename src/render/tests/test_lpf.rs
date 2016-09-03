use std::f32;

use signal::Signal;
use render::render_spec::{RenderSpec, RenderSpecFactory};
use render::reference::tree_renderer::TreeRenderer;
use tree::node::{Node, NodeInputSlot, NodeOp};
use tree::send::Send;
use tree::tree::Tree;

use super::approx_equal::assert_similar_audio;

// Render both a 440 Hz and 880 Hz wave after passing them through a LPF
// The LPF transfer function is given by:
// A = cos(w*0.0002)
// We accomplish this via two delays:
// cos(w*t+p)*cos(w*a) = 0.5 [cos(w*t+p+w*a) + cos(w*t+p-w*a)]
//  = 0.5 [cos(w*(t+a)+p) + cos(w*(t-a)+p)]
pub fn get_lpf(render_spec: RenderSpec, n_samples : u32) -> Vec<f32> {
    let mut tree = TreeRenderer::new(render_spec);
    let exit_node = Node::new_rc(NodeOp::OpAt);
    tree.watch_nodes(&vec![exit_node.clone()]);
    

    // Tree looks like:
    // enter_ynode  enter_05_node   enter_anode  enter_pos1_node
    //     \          /      _______|   ____|________|
    //      half_ynode      |          |    |
    //              \       |    neg_autom_node
    //               |      |______/
    //               |      |
    //               exit_node

    // create nodes for the above tree
    let enter_05_node = Node::default_rc();
    let enter_ynode = Node::default_rc();
    let enter_anode = Node::default_rc();
    let enter_pos1_node = Node::default_rc();
    let half_ynode = Node::new_rc(NodeOp::OpBy);
    let neg_autom_node = Node::new_rc(NodeOp::OpBy);

    // Link nodes together as depicted in tree:
    tree.add_send(
        Send::new_nodesend(enter_05_node.clone(), half_ynode.clone(), NodeInputSlot::Right)
    );
    tree.add_send(
        Send::new_nodesend(enter_ynode.clone(), half_ynode.clone(), NodeInputSlot::Left)
    );
    tree.add_send(
        Send::new_nodesend(enter_anode.clone(), neg_autom_node.clone(), NodeInputSlot::Right)
    );
    tree.add_send(
        Send::new_nodesend(enter_pos1_node.clone(), neg_autom_node.clone(), NodeInputSlot::Left)
    );
    tree.add_send(
        Send::new_nodesend(half_ynode.clone(), exit_node.clone(), NodeInputSlot::Left)
    );
    tree.add_send(
        Send::new_nodesend(enter_anode.clone(), exit_node.clone(), NodeInputSlot::Right)
    );
    tree.add_send(
        Send::new_nodesend(neg_autom_node.clone(), exit_node.clone(), NodeInputSlot::Right)
    );

    // connect constant inputs:
    tree.add_send(
        Send::new_srcsend(Signal::new(0.5f32, 0f32, 0f32, 0f32, -1f32), enter_05_node.clone())
    );
    tree.add_send(
        Send::new_srcsend(Signal::new(1.0f32, 0f32, 0f32, 0f32, 0f32), enter_pos1_node.clone())
    );


    // Send the stimulus (440, 880 Hz cosine). Begin at t=-1 to combat delay effect.
    tree.add_send(
        Send::new_srcsend(Signal::new(1.0f32, 440.0*2.0*f32::consts::PI, 0.5*f32::consts::PI, 0f32, -1f32), enter_ynode.clone())
    );
    tree.add_send(
        Send::new_srcsend(Signal::new(1.0f32, 880.0*2.0*f32::consts::PI, 0.5*f32::consts::PI, 0f32, -1f32), enter_ynode.clone())
    );

    // Send the filter coefficient
    tree.add_send(
        Send::new_srcsend(Signal::new(1.0f32, 0f32, 0f32, 0.0002f32, -1f32), enter_anode.clone())
    );


    let mut samples = vec![];
    for _ in 0..n_samples {
        samples.push(tree.step()[0]);
    }
    samples
}

#[test]
pub fn test_lpf() {
    let render_spec = RenderSpecFactory::new().sample_rate(44100).finalize();
    let n_samples = 100;
    let actual = get_lpf(render_spec, n_samples);
    let mut reference = vec![];

    let w_440 = 440f32*2f32*f32::consts::PI;
    let coeff_440 = (w_440*0.0002).cos();
    let w_880 = 880f32*2f32*f32::consts::PI;
    let coeff_880 = (w_880*0.0002).cos();

    for i in 0..n_samples {
        let t = (i as f32) / 44100f32;
        reference.push(
            coeff_440*(w_440*t).sin() +
            coeff_880*(w_880*t).sin()
        );
    }

    assert_similar_audio(&reference, &actual);
}

