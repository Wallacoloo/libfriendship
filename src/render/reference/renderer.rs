use std::collections::HashMap;

use render::renderer::Renderer;
use routing::{RouteTree, WeakNodeHandle};

struct RefRenderer {
    // associate some rendering state with each node.
    states: HashMap<WeakNodeHandle, NodeState>,
}

struct NodeState {
    // We track N+k output samples for each node, where N is the latency / block size
    // and k is the maximum that any input signal could be delayed by.
    // Externally, only the first N samples are visible.
    output: Vec<f32>,
}

impl Renderer for RefRenderer {
    fn step(&mut self, tree: &RouteTree, into: &mut [f32]) {
        // iterate from leaves up to the root.
        for node_handle in tree.iter_topo_rev() {
            self.states.entry(node_handle.weak()).or_insert_with(NodeState::new);
            unimplemented!();
        }
    }
}


impl NodeState {
    fn new() -> Self {
        NodeState {
            output: Vec::new()
        }
    }
}
