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

// Render both a 440 Hz and 880 Hz wave after passing them through a LPF
// The LPF transfer function is given by:
// A = cos(w*0.0002)
// Given e^(iw) = cos(w) + i*sin(w),
// the automation should be: 0.5exp(0.0002*ww) + 0.5exp(-0.0002*ww)
pub fn get_lpf(render_spec: RenderSpec, n_samples : u32) -> Vec<f32> {
    let mut tree = TreeRenderer::new(render_spec);
    let exit_node = YNode::new_rc();
    tree.watch_nodes(&vec![exit_node.clone()]);
    
    // create nodes for broadcasting the partials & automations
    let enter_ynode = YNode::new_rc();
    tree.add_send(
        Send::new_yysend(enter_ynode.clone(), exit_node.clone())
    );
    let enter_anode = ANode::new_rc();
    tree.add_send(
        Send::new_aysend(enter_anode.clone(), exit_node.clone())
    );

    // inject the automations
    tree.add_send(
        Send::new_asrcsend(
            Automation::new(PhaserCoeff::new_f32(0.5f32, 0f32), Real32::new(0f32), Real32::new(0.0002f32)),
            enter_anode.clone()
        )
    );
    tree.add_send(
        Send::new_asrcsend(
            Automation::new(PhaserCoeff::new_f32(0.5f32, 0f32), Real32::new(0f32), Real32::new(-0.0002f32)),
            enter_anode.clone()
        )
    );

    // inject a 440Hz and 880Hz sine wave
    tree.add_send(
        Send::new_ysrcsend(
            Partial::new(PhaserCoeff::new_f32(0f32, -1f32), Real32::new(440f32*2f32*f32::consts::PI)),
            enter_ynode.clone()
        )
    );
    tree.add_send(
        Send::new_ysrcsend(
            Partial::new(PhaserCoeff::new_f32(0f32, -1f32), Real32::new(880f32*2f32*f32::consts::PI)),
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
pub fn test_lpf() {
    let render_spec = RenderSpecFactory::new().sample_rate(44100).finalize();
    let n_samples = 100;
    let actual = get_lpf(render_spec, n_samples);
    let mut reference = vec![];

    let w_440 = 440f32*2f32*f32::consts::PI;
    let coeff_440 = (w_440*0.0002).cos();
    let w_880 = 880f32*2f32*f32::consts::PI;
    let coeff_880 = (w_880*0.0002).cos();

    for i in (0..n_samples) {
        let t = (i as f32) / 44100f32;
        reference.push(
            coeff_440*(w_440*t).sin() +
            coeff_880*(w_880*t).sin()
        );
    }

    assert_similar_audio(&reference, &actual);
}

