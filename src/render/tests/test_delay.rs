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

// Render both a 440 Hz and 880 Hz wave after passing them through a 1 ms delay
// Note: @44100 fps, 1 ms = 44.1 frames
// Note: if Y = c1 exp(i wty t)
// and A = c2 exp(i ww wty) [no exp(i*wta)]
// we desire A*Y = c1 c2 exp(i wty (t - t0))
// A*Y = c1 c2 exp(i wty t) exp(- i wty t0)
// Therefore, ww should be -t0
pub fn get_delay(render_spec: RenderSpec, n_samples : u32) -> Vec<f32> {
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
            Automation::new(PhaserCoeff::new_f32(1.0f32, 0f32), Real32::new(0f32), Real32::new(-0.001f32)),
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
pub fn test_delay() {
    let render_spec = RenderSpecFactory::new().sample_rate(44100).finalize();
    let n_samples = 100;
    let actual = get_delay(render_spec, n_samples);
    let mut reference = vec![];

    let w_440 = 440f32*2f32*f32::consts::PI;
    let w_880 = 880f32*2f32*f32::consts::PI;

    for i in (0..n_samples) {
        let t = (i as f32) / 44100f32 - 0.001;
        reference.push((w_440*t).sin() + (w_880*t).sin());
    }

    assert_similar_audio(&reference, &actual);
}

