use std::f32;

use routing::{LeafNode, PwLine, RouteEdge, RouteNode, RouteGraph, Sinusoid};
use render::renderer::Renderer;
use render::reference::renderer::RefRenderer;
use super::approx_equal::assert_similar_audio;

#[test]
/// Create a RouteGraph that outputs a triangle wave w/ const. amplitude
pub fn test_tri() {
    // fundamental freq of tri wave
    let phase_delta = 0.1f32;
    // number of harmonics (f0 is part of this count)
    let n_harm = 64;
    let pi = f32::consts::PI;
    let mut tree = RouteGraph::new();

    for k in 0..n_harm {
        let h_no = (2*k+1) as f32;
        // According to wikipedia, each harmonic is:
        // 8/pi^2 * (-1)^k*sin((2*k+1)*f*t))/(2k+1)^2
        let amp = 8f32/(pi*pi*h_no*h_no) *
            if k%2 == 0 { 1f32 } else { -1f32 };
        let my_phase_delta = h_no*phase_delta;
        let phase_0 = 0f32;

        let sin = Sinusoid::from_phase_info(phase_0, my_phase_delta);
        let pwline = PwLine::from_const(amp);
        let leaf_sin    = tree.add_node(RouteNode::new_leaf(LeafNode::new_sinusoid(sin)));
        let leaf_pwline = tree.add_node(RouteNode::new_leaf(LeafNode::new_pwline(pwline)));
        let output_sine = tree.add_node(RouteNode::new_intermediary());

        // route the leaves to the output sine
        tree.add_edge(&output_sine, &leaf_sin, RouteEdge::new_left());
        tree.add_edge(&output_sine, &leaf_pwline, RouteEdge::new_right(0));

        // Route the sine to ch0 output.
        tree.make_channel_output(&output_sine, 0);
    }

    // Compute expected outputs
    let mut expecting = vec![];
    for i in 0..100 {
        let if32 = i as f32;
        // We expect 0 at t=0, 1 at t=0.5pi/phase_delta,
        // 0 at t=pi/phase_delta, -1 at t=1.5pi/phase_delta
        let t = if32*phase_delta % (2.0f32*pi);
        let exp = if t < 0.5f32*pi {
            // line from (0, 0) to (pi/2, 1)
            t / (0.5f32*pi)
        } else if t < 1.5f32*pi {
            // line from (0.5pi, 1) to (1.5pi, -1)
            (pi-t) / (0.5f32*pi)
        } else {
            (t-2.0f32*pi) / (0.5f32*pi)
        };
        expecting.push(exp);
    }

    // Do the rendering
    let mut rend = RefRenderer::new();
    let mut buffer = vec![];
    buffer.resize(100, 0.0f32);
    rend.step(&tree, &mut buffer);

    assert_similar_audio(&expecting, &buffer);
}
