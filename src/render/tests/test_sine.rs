use routing::{LeafNode, RouteNode, RouteTree, Sinusoid};
use render::renderer::Renderer;
use render::reference::renderer::RefRenderer;
use super::approx_equal::assert_similar_audio;

#[test]
/// Create a RouteTree that outputs a sinusoid w/ const. amplitude
pub fn test_sine() {
    // define frequency/phase of sinusoid
    let phase_0 = 1.0f32;
    let phase_delta = 0.1f32;
    let mut tree = RouteTree::new();

    let sin = Sinusoid::from_phase_info(phase_0, phase_delta);
    let leaf_sin = LeafNode::new_sinusoid(sin);
    // Configure ch0 = output of channel 0;
    let ch0 = tree.add_node(RouteNode::new_leaf(leaf_sin));
    tree.make_channel_output(&ch0, 0);

    // Compute expected outputs
    let mut expecting = vec![];
    for i in 0..100 {
        expecting.push((phase_0+(i as f32)*phase_delta).sin());
    }

    // Do the rendering
    let mut rend = RefRenderer::new();
    let mut buffer = vec![];
    buffer.resize(100, 0.0f32);
    rend.step(&tree, &mut buffer);

    assert_similar_audio(&expecting, &buffer);
}
