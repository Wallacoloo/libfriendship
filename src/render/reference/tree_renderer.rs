use std::collections::hash_map::HashMap;
use std::rc::Rc;

use signal::Signal;
use tree::node::{Node, NodeInputSlot};
use tree::send::{NodeSend, Send};
use tree::tree::Tree;

use render::render_spec::RenderSpec;

use super::partial_renderer::PartialRenderer;

/// Tracks all the signals that have ever entered the node.
/// This information is needed in order to evaluate the output whenever
/// a new input arrives.

struct NodeState {
    left: Vec<Signal>,
    right: Vec<Signal>,
    sends: Vec<NodeSend>,
}

struct OutputState {
    node: Rc<Node>,
    renderer: PartialRenderer,
}

pub struct TreeRenderer {
    render_spec: RenderSpec,
    node_states: HashMap<Rc<Node>, NodeState>,
    outputs: Vec<OutputState>,
    // In order to return a slice from the step() function, we need to allocate
    // some buffer space
    output_buff: Vec<f32>,
}

impl TreeRenderer {
    pub fn new(render_spec: RenderSpec) -> TreeRenderer {
        TreeRenderer{
            render_spec: render_spec,
            node_states: HashMap::new(),
            outputs: vec![],
            output_buff: vec![],
        }
    }
    /// Get the NodeState associated with a node,
    /// and create it if it doesn't already exist.
    fn get_node_state(&mut self, node: &Rc<Node>) -> &mut NodeState {
        self.node_states.entry(node.clone()).or_insert_with(|| NodeState::new())
    }
    /// Send a signal to the input of any nodes connected to the output of the
    /// given node.
    fn broadcast_signal(&mut self, from: &Rc<Node>, signal: Signal) {
        // need to clone the State's sends to avoid mutability problems
        let sends = self.get_node_state(from).sends.clone();
        for ref send in sends.iter() {
            self.node_input(send.dest(), send.dest_slot(), signal);
        }
        // If we are operating on an output node, send this to the PartialRenderer
        if let Some(output) =
            self.outputs.iter_mut().find(|output_state|
                output_state.node() == from
            ) {
            output.feed(signal);
        }
    }
    fn node_input(&mut self, node: &Rc<Node>, slot: NodeInputSlot,
      signal: Signal) {
        match slot {
            NodeInputSlot::Left => {
                let right;
                {
                    let mut state = self.get_node_state(node);
                    state.left.push(signal);
                    right = state.right.clone();
                }
                // all the signals in the right bin affect the new signal
                for sig_right in right.iter() {
                    for new_sig in sig_right.apply_to_left(&signal, node.op()).iter() {
                        self.broadcast_signal(node, *new_sig);
                    }
                }
            },
            NodeInputSlot::Right => {
                let left;
                {
                    let mut state = self.get_node_state(node);
                    state.right.push(signal);
                    left = state.left.clone();
                }
                // this new signal affects all signals in the left bin
                for sig_left in left.iter() {
                    for new_sig in signal.apply_to_left(sig_left, node.op()).iter() {
                        self.broadcast_signal(node, *new_sig);
                    }
                }
            },
        };
    }
}

impl Tree for TreeRenderer {
    /// Connect two nodes in the tree.
    /// If the send is SrcSend, this is the same as feeding
    /// external stimuli into the tree.
    fn add_send(&mut self, send: Send) {
        match send {
            Send::NodeSend(send) => {
                self.get_node_state(send.src()).add_nodesend(send);
            },
            Send::SrcSend(send) => {
                self.broadcast_signal(send.dest(), *send.src());
            },
        }
    }

    /// set the nodes for which we are interested in the output PCM signals.
    /// Future calls to `step()` will return an array of samples corresponding
    /// to these nodes.
    fn watch_nodes(&mut self, outputs: &[Rc<Node>]) {
        self.outputs = outputs.iter().map(|node| 
            OutputState::new(self.render_spec.clone(), node.clone())
        ).collect();
    }
    /// Return the next buffer of samples related to the watched nodes.
    fn step(&mut self) -> &[f32] {
        // Todo: Make use of `Vec::resize once stabilized (Projected for Rust 1.5)
        self.output_buff = self.outputs.iter_mut().map(
            |output| output.step()
        ).collect();
        &self.output_buff
    }
}


impl NodeState {
    fn new() -> NodeState {
        NodeState{ left:vec![], right: vec![], sends: vec![] }
    }
    fn add_nodesend(&mut self, send: NodeSend) {
        self.sends.push(send);
    }
}


impl OutputState {
    fn new(spec: RenderSpec, node: Rc<Node>) -> OutputState {
        OutputState{ node: node, renderer: PartialRenderer::new(spec) }
    }
    fn node(&self) -> &Rc<Node> {
        &self.node
    }
    fn step(&mut self) -> f32 {
        self.renderer.step()
    }
    fn feed(&mut self, signal: Signal) {
        self.renderer.feed(signal);
    }
}

