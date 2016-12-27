use routing::{LeafNode, PwLine, RouteEdge, RouteNode, RouteTree};
use render::renderer::Renderer;
use render::reference::renderer::RefRenderer;
use super::approx_equal::assert_similar_audio;

#[test]
/// Create a RouteTree that just outputs a constant value.
pub fn test_const() {
    // We multiply cval*cval
    let cval = 1.2f32;
    let cval_prod = 1.44f32;
    let mut tree = RouteTree::new();
    let mut pw_const = PwLine::new();

    // Configure ch0 = output of channel 0;
    let ch0 = tree.add_node(RouteNode::new_intermediary());
    tree.make_channel_output(&ch0, 0);

    // create a line that is really just a constant.
    pw_const.add_pt(0u32, cval);
    pw_const.add_pt(44100u32, cval);
    // Route it to both sides of ch0
    let leaf_a = LeafNode::new_pwline(pw_const.clone());
    let leaf_b = LeafNode::new_pwline(pw_const);
    let hdl_a = tree.add_node(RouteNode::new_leaf(leaf_a));
    let hdl_b = tree.add_node(RouteNode::new_leaf(leaf_b));
    tree.add_edge(&ch0, &hdl_a, RouteEdge::new_left());
    tree.add_edge(&ch0, &hdl_a, RouteEdge::new_right(0));

    // Compute expected outputs
    let mut expecting = vec![];
    expecting.resize(100, cval_prod);

    // Do the rendering
    let mut rend = RefRenderer::new();
    let mut buffer = vec![];
    buffer.resize(100, 0.0f32);
    rend.step(&tree, &mut buffer);

    assert_similar_audio(&expecting, &buffer);
}
