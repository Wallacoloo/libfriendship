use std::f32;

use automation::Automation;
use partial::Partial;
use phaser::PhaserCoeff;
use real::Real32;
use render::render_spec::{RenderSpec, RenderSpecFactory};
use render::reference::tree_renderer::TreeRenderer;
use tree::node::{ANode, YNode};
use tree::send::Send;
use tree::tree::Tree;

use super::approx_equal::assert_similar_audio;

/// Gate a 440 Hz wave by u(t)
/// u(t) is approximated via the fourier series of a square wave.
pub fn get_gate(render_spec: RenderSpec, n_samples : u32) -> Vec<f32> {
    let mut tree = TreeRenderer::new(render_spec);
    let exit_node = YNode::new_rc();
    tree.watch_nodes(&vec![exit_node.clone()]);

    // The tree looks like:
    // enter_ynode enter_gate_node
    //         \       /
    //         exit_node
    
    // create nodes for broadcasting the partials & automations
    let enter_ynode = YNode::new_rc();
    tree.add_send(
        Send::new_yysend(enter_ynode.clone(), exit_node.clone())
    );
    let enter_gate_anode = ANode::new_rc();
    tree.add_send(
        Send::new_aysend(enter_gate_anode.clone(), exit_node.clone())
    );

    // inject the automations
    // Note: for volume gating, we only use the c2 exp(i*wt*t) portion of the
    // automations
    // Note: first, add the 0.5 DC offset:
    tree.add_send(
        Send::new_asrcsend(
            Automation::new(PhaserCoeff::new_f32(0.5, 0.0), Real32::new(0.0), Real32::new(0.0)),
            enter_gate_anode.clone()
        )
    );
    for i in (1..400) {
        let harmonic = (2*i - 1) as f32;
        let amp = 0.25*4.0/f32::consts::PI/harmonic;
        let freq = 100.0*2.0*f32::consts::PI * harmonic;
        tree.add_send(
            Send::new_asrcsend(
                Automation::new(PhaserCoeff::new_f32(0.0, -amp), Real32::new(freq), Real32::new(0.0)),
                enter_gate_anode.clone()
            )
        );
        tree.add_send(
            Send::new_asrcsend(
                Automation::new(PhaserCoeff::new_f32(0.0, amp), Real32::new(-freq), Real32::new(0.0)),
                enter_gate_anode.clone()
            )
        );
    }

    // inject a 440Hz sine wave
    tree.add_send(
        Send::new_ysrcsend(
            Partial::new(PhaserCoeff::new_f32(0.0, -1.0), Real32::new(440.0*2.0*f32::consts::PI)),
            enter_ynode.clone()
        )
    );

    let mut samples = vec![];
    for _ in (0..n_samples) {
        samples.push(tree.step()[0]);
    }
    samples
}

#[test]
pub fn test_gate() {
    let render_spec = RenderSpecFactory::new().sample_rate(44100).finalize();
    let n_samples = 100;
    let actual = get_gate(render_spec, n_samples);
    let mut reference = vec![];

    let w_440 = 440.0*2.0*f32::consts::PI;

    for i in (0..n_samples) {
        let t = (i as f32) / 44100.0;
        reference.push((w_440*t).sin());
    }

    assert_similar_audio(&reference, &actual);
}

