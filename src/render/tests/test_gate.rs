use std::f32;

use signal::Signal;
use render::render_spec::{RenderSpec, RenderSpecFactory};
use render::reference::tree_renderer::TreeRenderer;
use tree::node::{Node, NodeInputSlot};
use tree::send::Send;
use tree::tree::Tree;

use super::approx_equal::assert_similar_audio;


/// Gate a 440 Hz wave by (u(t) - u(t-0.005))
pub fn get_gate(render_spec: RenderSpec, n_samples : u32) -> Vec<f32> {
    let mut tree = TreeRenderer::new(render_spec);
    let exit_node = Node::default_rc();
    tree.watch_nodes(&vec![exit_node.clone()]);

    // The tree looks like:
    // enter_ynode enter_gate_node
    //         \       /
    //         mult_node
    
    // create nodes for broadcasting the partials & automations
    let enter_ynode = Node::default_rc();
    tree.add_send(
        Send::new_nodesend(enter_ynode.clone(), exit_node.clone(), NodeInputSlot::Left)
    );
    let enter_gate_node = Node::default_rc();
    tree.add_send(
        Send::new_nodesend(enter_gate_node.clone(), exit_node.clone(), NodeInputSlot::Right)
    );

    // inject the automations
    // +1 amplitude gated at 0 and -1 amplitude gated at 0.005
    tree.add_send(
        Send::new_srcsend(
            Signal::new(1.0, 0.0, 0.0, 0.0, 0.0),
            enter_gate_node.clone()
        )
    );
    tree.add_send(
        Send::new_srcsend(
            Signal::new(1.0, 0.0, f32::consts::PI, 0.0, 0.005),
            enter_gate_node.clone()
        )
    );

    // inject a 440Hz sine wave
    tree.add_send(
        Send::new_srcsend(
            Signal::new(1.0, 440.0*2.0*f32::consts::PI, 0.5*f32::consts::PI, 0.0, 0.0),
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
pub fn test_gate() {
    let render_spec = RenderSpecFactory::new().sample_rate(44100).finalize();
    let n_samples = 200;
    let actual = get_gate(render_spec, n_samples);
    let mut reference = vec![];

    let w_440 = 440.0*2.0*f32::consts::PI;

    for i in 0..n_samples {
        let t = (i as f32) / 44100.0;
        reference.push(
            if t <= 0.5/220.0 {
                (w_440*t).sin()
            } else {
                0.0
            }
        );
    }

    assert_similar_audio(&reference, &actual);
}

