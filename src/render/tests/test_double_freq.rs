use std::f32;

use signal::Signal;
use render::render_spec::{RenderSpec, RenderSpecFactory};
use render::reference::tree_renderer::TreeRenderer;
use tree::node::{Node, NodeInputSlot, NodeOp};
use tree::send::Send;
use tree::tree::Tree;

use super::approx_equal::assert_similar_audio;

/// Creates a 440 Hz sine wave and then generates the first harmonic via:
/// cos(w*t)*cos(w*t) = 0.5 [cos(2*w*t) + cos(0)] = 0.5 + 0.5*cos(2*w*t)
/// Therefore, cos(2*w*t) = 2*cos(w*t)*cos(w*t) - 1
pub fn get_double_freq(render_spec: RenderSpec, n_samples : u32) -> Vec<f32> {
    let mut tree = TreeRenderer::new(render_spec);
    let exit_node = Node::new_rc(NodeOp::OpAt);
    tree.watch_nodes(&vec![exit_node.clone()]);
    
    // create nodes for broadcasting the partials & automations
    let enter_ynode = Node::default_rc();
    let enter_pos2_node = Node::default_rc();
    let double_amp_node = Node::new_rc(NodeOp::OpAt);
    let mult_node = Node::new_rc(NodeOp::OpAt);
    let enter_neg1_node = Node::default_rc();
    let enter_pos1_node = Node::default_rc();

    // arrangement:
    // enter_ynode   enter_pos2_node
    //      |      \      /
    //      |    double_amp_node
    //      |   ______/
    //      |  |
    //    mult_node    enter_neg1_node
    //      |________________|    enter_pos1_node
    //      |    ________________________|
    //      |   |
    //    exit_node

    // Link all nodes together according to the above diagram:
    tree.add_send(
        Send::new_nodesend(enter_ynode.clone(), mult_node.clone(), NodeInputSlot::Left)
    );
    tree.add_send(
        Send::new_nodesend(enter_ynode.clone(), double_amp_node.clone(), NodeInputSlot::Left)
    );
    tree.add_send(
        Send::new_nodesend(enter_pos2_node.clone(), double_amp_node.clone(), NodeInputSlot::Right)
    );
    tree.add_send(
        Send::new_nodesend(double_amp_node.clone(), mult_node.clone(), NodeInputSlot::Right)
    );
    tree.add_send(
        Send::new_nodesend(mult_node.clone(), exit_node.clone(), NodeInputSlot::Left)
    );
    tree.add_send(
        Send::new_nodesend(enter_neg1_node.clone(), exit_node.clone(), NodeInputSlot::Left)
    );
    tree.add_send(
        Send::new_nodesend(enter_pos1_node.clone(), exit_node.clone(), NodeInputSlot::Right)
    );
    // connect the constant inputs:
    tree.add_send(
        Send::new_srcsend(Signal::new(2.0f32, 0f32, 0f32, 0f32, 0f32), enter_pos2_node.clone())
    );
    tree.add_send(
        Send::new_srcsend(Signal::new(-1.0f32, 0f32, 0f32, 0f32, 0f32), enter_neg1_node.clone())
    );
    tree.add_send(
        Send::new_srcsend(Signal::new(1.0f32, 0f32, 0f32, 0f32, 0f32), enter_pos1_node.clone())
    );

    // Send the stimulus (440 Hz cosine)
    tree.add_send(
        Send::new_srcsend(Signal::new(1.0f32, 440.0*2.0*f32::consts::PI, 0f32, 0f32, 0f32), enter_ynode.clone())
    );

    let mut samples = vec![];
    for _ in 0..n_samples {
        samples.push(tree.step()[0]);
    }
    samples
}

#[test]
pub fn test_double_freq() {
    let render_spec = RenderSpecFactory::new().sample_rate(44100).finalize();
    let n_samples = 200;
    let actual = get_double_freq(render_spec, n_samples);
    let mut reference = vec![];

    let w_880 = 880.0*2.0*f32::consts::PI;

    for i in 0..n_samples {
        let t = (i as f32) / 44100.0;
        reference.push((w_880*t).cos());
    }

    assert_similar_audio(&reference, &actual);
}

