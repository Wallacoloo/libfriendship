use routing::{LeafNode, PwLine, RouteEdge, RouteNode, RouteTree};
use render::renderer::Renderer;
use render::reference::renderer::RefRenderer;
use super::approx_equal::assert_similar_audio;

#[test]
/// Create a RouteTree that takes a square wave and passes it through a high-pass filter.
pub fn test_hpf() {
    // We multiply cval*cval
    let mut tree = RouteTree::new();
    let mut pw_square = PwLine::new();
    // 4 iterations of a two-frame period square wave.
    for i in 0u32..4u32 {
        pw_square.add_pt(0u32 +i*20u32, -1.0f32);
        pw_square.add_pt(9u32 +i*20u32, -1.0f32);

        pw_square.add_pt(10u32+i*20u32, 1.0f32);
        pw_square.add_pt(19u32+i*20u32, 1.0f32);
    }

    // Configure ch0 = output of channel 0;
    let ch0 = tree.add_node(RouteNode::new_intermediary());
    tree.make_channel_output(&ch0, 0);

    // Route it to both sides of ch0
    let input = LeafNode::new_pwline(pw_square);
    // transfer function
    let h0 = LeafNode::new_pwline(PwLine::from_const(1.0f32));
    let h1 = LeafNode::new_pwline(PwLine::from_const(-1.0f32));
    let hdl_input = tree.add_node(RouteNode::new_leaf(input));
    let hdl_h0 = tree.add_node(RouteNode::new_leaf(h0));
    let hdl_h1 = tree.add_node(RouteNode::new_leaf(h1));
    tree.add_edge(&ch0, &hdl_input, RouteEdge::new_left());
    tree.add_edge(&ch0, &hdl_h0, RouteEdge::new_right(0));
    tree.add_edge(&ch0, &hdl_h1, RouteEdge::new_right(1));

    // Compute expected outputs
    let mut expecting = vec![];
    expecting.resize(80, 0f32);
    // at t=0, we effectively transition from 0 -> -1
    expecting[0] = -1.0f32;
    // at t=10 (mod 20), we transition from -1 -> 1; output = 2
    expecting[10] = 2.0f32;
    // at t=20 (mod 20), we transition from 1 -> -1; output = -2
    for i in 1..4 {
        expecting[i*20] = -2.0f32;
        expecting[10+i*20] = 2.0f32;
    }

    // Do the rendering
    let mut rend = RefRenderer::new();
    let mut buffer = vec![];
    buffer.resize(80, 0.0f32);
    rend.step(&tree, &mut buffer);

    assert_similar_audio(&expecting, &buffer);
}
