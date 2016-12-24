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
    let root = tree.root().clone();

    // create a line that is really just a constant.
    pw_const.add_pt(0u32, cval);
    pw_const.add_pt(44100u32, cval);
    // Route it to both sides of the root:
    let leaf_a = LeafNode::PwLine(pw_const.clone());
    let leaf_b = LeafNode::PwLine(pw_const);
    let hdl_a = tree.add_node(RouteNode::Leaf(leaf_a));
    let hdl_b = tree.add_node(RouteNode::Leaf(leaf_b));
    tree.add_edge(&hdl_a, &root, RouteEdge::new_left());
    tree.add_edge(&hdl_a, &root, RouteEdge::new_right(0));

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
