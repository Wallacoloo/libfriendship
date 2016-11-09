use render::renderer::Renderer;
use routing::RouteTree;

struct RefRenderer {
}

impl Renderer for RefRenderer {
    fn step(&mut self, tree: &RouteTree, into: &mut [f32]) {
        // iterate from leaves up to the root.
        for node_handle in tree.iter_topo_rev() {
            unimplemented!();
        }
    }
}
