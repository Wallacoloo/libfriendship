use std::collections::HashMap;

use render::renderer::Renderer;
use routing::{RouteTree, WeakNodeHandle};

pub struct RefRenderer {
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
    #[allow(non_snake_case)]
    fn step(&mut self, tree: &RouteTree, into: &mut [f32]) {
        // save the buffer size, for use throughout.
        let N = into.len();
        // iterate from leaves up to the root.
        for node_handle in tree.iter_topo_rev() {
            // Create temporary state for each of our inputs (note: multiple sources could sum into
            // the same slot)
            let mut in_buffs = Vec::new();
            // Now we gather N samples from each child & bring them in.
            for edge in tree.children_of(&node_handle) {
                let slot = edge.weight().slot_idx();
                let child_state = self.states.get(&edge.to().weak()).unwrap();
                // Create a buffer for this slot if not yet created.
                if slot >= in_buffs.len() {
                    in_buffs.resize(slot, vec![0f32; N].into_boxed_slice());
                }
                // sum the child's output into the correct buffer.
                for (into, from_child) in in_buffs[slot].iter_mut().zip(child_state.get(N)) {
                    *into = *from_child;
                }
            }
            // Create a state entry for the node
            let mut state = self.states.entry(node_handle.weak()).or_insert_with(NodeState::new);
            // we need enough room to house a length-N sequence that's delayed by in_buffs.len()-2
            state.ensure_len(N + in_buffs.len()-2);
            // Sum the convolutions of in_buffs into the state's output:
            // i.e. output = \sum{i=1}^m in_buffs[0][t] \conv in_buffs[i][t - (i+1)]
            for (i, buff) in in_buffs.iter().enumerate().skip(1) {
                state.sum_into(buff, i-1);
            }
        }
        // Copy the output of the root node into our output buffer
        {
            let root_state = self.states.get(&tree.root().weak()).unwrap();
            into[..].clone_from_slice(root_state.get(N));
        }
        // Go back and reset each node's buffer
        for node_handle in tree.iter_topo_rev() {
            let mut state = self.states.get_mut(&node_handle.weak()).unwrap();
            state.advance_buff(N);
        }
    }
}


impl NodeState {
    fn new() -> Self {
        NodeState {
            output: Vec::new()
        }
    }
    /// get the head of the buffer
    fn get(&self, size: usize) -> &[f32] {
        &self.output[..size]
    }
    /// After processing the output, we can remove the head of the buffer
    fn advance_buff(&mut self, size: usize) {
        self.output.drain(..size);
    }
    fn ensure_len(&mut self, size: usize) {
        if self.output.len() < size {
            self.output.resize(size, 0f32);
        }
    }
    /// Add `data` into self.output[offset..]
    fn sum_into(&mut self, data: &[f32], offset: usize) {
        for (mine, other) in self.output.iter_mut().skip(offset).zip(data.iter()) {
            *mine += *other;
        }
    }
}
