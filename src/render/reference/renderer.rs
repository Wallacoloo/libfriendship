use std::collections::HashMap;

use render::renderer::Renderer;
use routing::{RouteNode, RouteGraph, WeakNodeHandle};

pub struct RefRenderer {
    // associate some rendering state with each node.
    states: HashMap<WeakNodeHandle, NodeState>,
    // index of the current sample being processed.
    sample_idx: u32,
}

struct NodeState {
    // We track 1+k output samples for each node, where 1 is the latency / block size
    // and k is the maximum that any input signal could be delayed by.
    // Externally, only the first sample is visible.
    //
    // Then, when a new set of inputs arrive (i.e. the left and right components at t=t0),
    // we can compute their full effect into the buffer.
    //
    // We could have a latency of N>1, but this complicates dependency tracking,
    // as cycles with delay N>1 ARE allowed.
    output: Vec<f32>,
}

impl Renderer for RefRenderer {
    /// Repeatedly step the tree to fill the buffer.
    /// If the buffer size is not a multiple of the channel count, the last incomplete frame will
    /// be left untouched.
    fn step(&mut self, tree: &EffectGraph, into: &mut [f32]) {
        let n_ch = tree.n_channels() as usize;
        for frame_no in 0..into.len()/n_ch {
            let buff_idx = frame_no*n_ch;
            self.step_once(tree, &mut into[buff_idx..buff_idx+n_ch]);
            self.sample_idx += 1;
        }
    }
    /// Called whenever a new edge is added to the EffectGraph of interest
    fn edge_added(&mut self, edge: EffectEdge) {
    }
    /// Called whenever an edge is removed from the EffectGraph of interest
    fn edge_removed(&mut self, edge: EffectEdge) {
    }
}

impl RefRenderer {
    pub fn new() -> Self {
        RefRenderer{
            states: HashMap::new(),
            sample_idx: 0,
        }
    }
    /// Repeatedly step the graph to fill the buffer.
    /// If the buffer size is not a multiple of the channel count, the last incomplete frame will
    /// be left untouched.
    fn step_routegraph(&mut self, tree: &RouteGraph, into: &mut [f32]) {
        let n_ch = tree.n_channels() as usize;
        for frame_no in 0..into.len()/n_ch {
            let buff_idx = frame_no*n_ch;
            self.step_once(tree, &mut into[buff_idx..buff_idx+n_ch]);
            self.sample_idx += 1;
        }
    }
    fn step_once(&mut self, tree: &RouteGraph, into: &mut [f32]) {
        // iterate from leaves up to the root.
        for node_handle in tree.iter_topo_rev() {
            match node_handle.node_data() {
                RouteNode::Intermediary => {
                    // Create temporary state for each of our inputs (note: multiple sources could sum into
                    // the same slot)
                    let mut in_buffs = Vec::new();
                    // Now we gather 1 sample from each child & bring it in.
                    for edge in tree.children_of(&node_handle) {
                        let slot = edge.weight().slot_idx() as usize;
                        let child_state = self.states.get(&edge.to().weak()).unwrap();
                        // Create a buffer for this slot if not yet created.
                        if slot >= in_buffs.len() {
                            in_buffs.resize(slot+1, 0f32);
                        }
                        // sum the child's output into the correct buffer.
                        in_buffs[slot] += child_state.head();
                    }
                    // get/create the state entry for the node
                    let mut state = self.states.entry(node_handle.weak()).or_insert_with(NodeState::new);
                    // Do the convolution, summing it into our state/output.
                    if in_buffs.len() > 0 {
                        let left_val = in_buffs[0];
                        for (right_idx, right_val) in in_buffs.into_iter().skip(1).enumerate() {
                            state.sum_into(left_val*right_val, right_idx);
                        }
                    }
                },
                RouteNode::Leaf(leaf_node) => {
                    // get/create the state entry for the node
                    let mut state = self.states.entry(node_handle.weak()).or_insert_with(NodeState::new);
                    state.sum_into(leaf_node.get_one(self.sample_idx), 0)
                }
            }
        }
        // Zero the output buffer so we can sum into it.
        for f in into.iter_mut() {
            *f = 0f32;
        }
        // Copy the right INPUTS into the root node into our output buffer
        //  (yes, we did compute the "output" of the root node above, but doing so was pointless).
        for edge in tree.right_children_of(&tree.root()) {
            let ch_no = edge.weight().delay();
            let child_state = self.states.get(&edge.to().weak()).unwrap();
            let sample = child_state.head();
            // write the sample to the output
            into[ch_no as usize] += sample;
        }
        // Go back and reset each node's buffer
        for node_handle in tree.iter_topo_rev() {
            let mut state = self.states.get_mut(&node_handle.weak()).unwrap();
            state.pop_head();
        }
    }
}


impl NodeState {
    fn new() -> Self {
        NodeState {
            output: Vec::new()
        }
    }
    /// get the head of the buffer, defaulting to 0.0
    fn head(&self) -> f32 {
        *self.output.iter().next().unwrap_or(&0.0f32)
    }
    /// After processing the output, we can remove the head of the buffer
    fn pop_head(&mut self) {
        if self.output.len() > 0 {
            self.output.remove(0);
        }
    }
    /// Add `data` into self.output[offset]
    fn sum_into(&mut self, data: f32, offset: usize) {
        if self.output.len() < offset+1 {
            self.output.resize(offset+1, 0f32);
        }
        self.output[offset] += data;
    }
}
