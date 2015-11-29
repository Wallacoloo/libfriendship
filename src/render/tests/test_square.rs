use std::f32;

use partial::Partial;
use phaser::PhaserCoeff;
use real::Real32;
use render::render_spec::{RenderSpec, RenderSpecFactory};
use render::reference::tree_renderer::TreeRenderer;
use tree::node::YNode;
use tree::send::Send;
use tree::tree::Tree;

use super::approx_equal::assert_similar_audio;

/// sinc math function: sin(pi x)/(pi x)
/// Often used in Sigma Approximation for Fourier series to make them converge
/// more rapidly.
fn sinc(x: f32) -> f32 {
    let pix = x * f32::consts::PI;
    pix.sin() / pix
}

/// return x.signum() if x != 0, else 0
fn trisign(x: f32, epsilon: f32) -> f32 {
    if x.abs() <= epsilon {
        0.0
    } else {
        x.signum()
    }
}

/// Try to render a single 441 Hz square wave through the reference tree_renderer
pub fn get_square(render_spec: RenderSpec, n_samples: u32) -> Vec<f32> {
    let mut tree = TreeRenderer::new(render_spec);
    let exit_node = YNode::new_rc();
    tree.watch_nodes(&vec![exit_node.clone()]);
    
    // inject the sine waves directly into the output
    for i in (1..101) {
        let harmonic = (i*2 - 1) as f32;
        let freq = 441.0*2.0*f32::consts::PI * harmonic;
        let amp = sinc((i as f32)/101.0) * 4.0 / f32::consts::PI / harmonic;
        tree.add_send(
            Send::new_ysrcsend(
                Partial::new(PhaserCoeff::new_f32(0.0, -amp), Real32::new(freq)),
                exit_node.clone()
            )
        );
    }


    let mut samples = vec![];
    for _ in (0..n_samples) {
        samples.push(tree.step()[0]);
    }
    samples
}


#[test]
pub fn test_square() {
    let render_spec = RenderSpecFactory::new().sample_rate(44100).finalize();
    let n_samples = 100;
    let actual = get_square(render_spec, n_samples);
    let mut reference = vec![];
    for i in (0..n_samples) {
        let t = (i as f32) / 44100.0;
        reference.push(trisign((441.0*2.0*f32::consts::PI * t).sin(), 0.000002));
    }

    assert_similar_audio(&reference, &actual);
}

