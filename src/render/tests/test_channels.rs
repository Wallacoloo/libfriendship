use routing::{LeafNode, PwLine, RouteEdge, RouteNode, RouteGraph, Sinusoid};
use render::renderer::Renderer;
use render::reference::renderer::RefRenderer;
use super::approx_equal::assert_similar_audio;

#[test]
/// Create a RouteGraph that outputs a sinusoid w/ const. amplitude to ch0
/// and a sinusoid of opposite amplitude to ch1
pub fn test_channels() {
    let phase_delta = 0.1f32;
    let mut tree = RouteGraph::new();

    let sin = Sinusoid::from_phase_info(0.0f32, phase_delta);
    let leaf_sin = LeafNode::new_sinusoid(sin);
    // Configure ch0 = output of channel 0;
    let ch0 = tree.add_node(RouteNode::new_leaf(leaf_sin));
    tree.make_channel_output(&ch0, 0);

    let leaf_invert = tree.add_node(
        RouteNode::new_leaf(LeafNode::new_pwline(PwLine::from_const(-1.0f32)))
    );
    let ch1 = tree.add_node(RouteNode::new_intermediary());
    tree.make_channel_output(&ch1, 1);
    // ch1 takes ch0 ...
    tree.add_edge(&ch1, &ch0, RouteEdge::new_left());
    // and inverts it.
    tree.add_edge(&ch1, &leaf_invert, RouteEdge::new_right(0));

    // Compute expected outputs
    let mut expecting = vec![];
    for i in 0..50 {
        let if32 = i as f32;
        expecting.push((phase_delta * if32).sin());
        expecting.push(-(phase_delta * if32).sin());
    }

    // Do the rendering
    let mut rend = RefRenderer::new();
    let mut buffer = vec![];
    buffer.resize(100, 0.0f32);
    rend.step(&tree, &mut buffer);

    assert_similar_audio(&expecting, &buffer);
}
