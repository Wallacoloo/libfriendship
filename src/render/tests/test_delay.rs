use std::f32;

use signal::Signal;
use render::render_spec::{RenderSpec, RenderSpecFactory};
use render::reference::tree_renderer::TreeRenderer;
use tree::node::{Node, NodeInputSlot, NodeOp};
use tree::send::Send;
use tree::tree::Tree;

use super::approx_equal::assert_similar_audio;

// Render both a 440 Hz and 880 Hz wave after passing them through a 1 ms delay
// Note: @44100 fps, 1 ms = 44.1 frames
// Note: if Y = c1 exp(i wty t)
// and A = c2 exp(i ww wty) [no exp(i*wta)]
// we desire A*Y = c1 c2 exp(i wty (t - t0))
// A*Y = c1 c2 exp(i wty t) exp(- i wty t0)
// Therefore, ww should be -t0
pub fn get_delay(render_spec: RenderSpec, n_samples : u32) -> Vec<f32> {
    let mut tree = TreeRenderer::new(render_spec);
    let exit_node = Node::new_rc(NodeOp::OpAt);
    tree.watch_nodes(&vec![exit_node.clone()]);
    
    // create nodes for broadcasting the partials & automations
    let enter_ynode = Node::new_rc(NodeOp::OpAt);
    tree.add_send(
        Send::new_nodesend(enter_ynode.clone(), exit_node.clone(), NodeInputSlot::Right)
    );
    let enter_anode = Node::new_rc(NodeOp::OpAt);
    tree.add_send(
        Send::new_nodesend(enter_anode.clone(), exit_node.clone(), NodeInputSlot::Left)
    );

    // inject the automations
    let autom = Signal::new(1.0, 0.0, 0.0, 0.001, 0.0);
    tree.add_send(
        Send::new_srcsend(
            autom,
            enter_anode.clone()
        )
    );

    // inject a 440Hz and 880Hz sine wave
    let y440 = Signal::new(1.0, 440.0*2.0*f32::consts::PI, 0.5*f32::consts::PI, 0.0, 0.0);
    let y880 = Signal::new(1.0, 880.0*2.0*f32::consts::PI, 0.5*f32::consts::PI, 0.0, 0.0);
    tree.add_send(
        Send::new_srcsend(
            y440,
            enter_ynode.clone()
        )
    );
    tree.add_send(
        Send::new_srcsend(
            y880,
            enter_ynode.clone()
        )
    );


    let mut samples = vec![];
    for _ in 0..n_samples {
        samples.push(tree.step()[0]);
    }
    samples
}

#[test]
pub fn test_delay() {
    let render_spec = RenderSpecFactory::new().sample_rate(44100).finalize();
    let n_samples = 100;
    let actual = get_delay(render_spec, n_samples);
    let mut reference = vec![];

    let w_440 = 440f32*2f32*f32::consts::PI;
    let w_880 = 880f32*2f32*f32::consts::PI;

    for i in 0..n_samples {
        let t = (i as f32) / 44100f32 - 0.001;
        reference.push((w_440*t).sin() + (w_880*t).sin());
    }

    assert_similar_audio(&reference, &actual);
}

