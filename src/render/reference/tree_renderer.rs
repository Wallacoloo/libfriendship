use std::collections::hash_map::HashMap;
use std::rc::Rc;

use automation::Automation;
use partial::Partial;
use tree::node::{ANode, NodeInputSlot, YNode};
use tree::send::{AASend, AYSend, Send, YYSend};
use tree::tree::Tree;

use render::render_spec::RenderSpec;

use super::partial_renderer::PartialRenderer;

/// Tracks all the partials/automations that have ever entered the node.
/// This information is needed in order to evaluate the output whenever
/// a new input arrives.
struct ANodeState {
    left: Vec<Automation>,
    right: Vec<Automation>,
    aasends: Vec<AASend>,
    aysends: Vec<AYSend>,
}

/// Tracks all the partials/automations that have ever entered the node.
/// This information is needed in order to evaluate the output whenever
/// a new input arrives.
struct YNodeState {
    left: Vec<Partial>,
    right: Vec<Automation>,
    yysends: Vec<YYSend>,
}

struct OutputState {
    node: Rc<YNode>,
    renderer: PartialRenderer,
}

struct TreeRenderer {
    render_spec: RenderSpec,
    anode_states: HashMap<Rc<ANode>, ANodeState>,
    ynode_states: HashMap<Rc<YNode>, YNodeState>,
    outputs: Vec<OutputState>,
    // In order to return a slice from the step() function, we need to allocate
    // some buffer space
    output_buff: Vec<f32>,
}

impl TreeRenderer {
    pub fn new(render_spec: RenderSpec) -> TreeRenderer {
        TreeRenderer{
            render_spec: render_spec,
            anode_states: HashMap::new(),
            ynode_states: HashMap::new(),
            outputs: vec![],
            output_buff: vec![],
        }
    }
    /// Get the ANodeState associated with a node,
    /// and create it if it doesn't already exist.
    fn get_anode_state(&mut self, node: &Rc<ANode>) -> &mut ANodeState {
        self.anode_states.entry(node.clone()).or_insert_with(|| ANodeState::new())
    }
    /// Get the YNodeState associated with a node,
    /// and create it if it doesn't already exist.
    fn get_ynode_state(&mut self, node: &Rc<YNode>) -> &mut YNodeState {
        self.ynode_states.entry(node.clone()).or_insert_with(|| YNodeState::new())
    }
    /// Send a partial to the input of any nodes connected to the output of the
    /// given node.
    fn broadcast_partial(&mut self, from: &Rc<YNode>, partial: Partial) {
        // need to clone the State's sends to avoid mutability problems
        let sends = self.get_ynode_state(from).yysends.clone();
        for ref send in sends.iter() {
            self.ynode_input_left(send.dest(), partial);
        }
        // If we are operating on an output node, send this to the PartialRenderer
        if let Some(output) =
            self.outputs.iter_mut().find(|output_state|
                output_state.node() == from
            ) {
            output.feed(partial);
        }
    }
    /// Send an Automation to the input of any nodes connected to the output of
    /// the given node.
    fn broadcast_autom(&mut self, from: &Rc<ANode>, autom: Automation) {
        // need to clone the State's sends to avoid mutability problems
        let aasends = self.get_anode_state(from).aasends.clone();
        let aysends = self.get_anode_state(from).aysends.clone();
        // send automation input to the YNodes:
        for ref send in aysends.iter() {
            self.ynode_input_right(send.dest(), autom);
        }
        // send the automation to the other ANodes:
        for ref send in aasends.iter() {
            self.anode_input(send.dest(), send.dest_slot(), autom);
        }
    }
    /// Add a partial to the YNode's left slot
    fn ynode_input_left(&mut self, node: &Rc<YNode>, partial: Partial) {
        let right;
        {
            let mut state = self.get_ynode_state(node);
            state.left.push(partial);
            // need to clone the State's right-hand-side to avoid mut issues
            right = state.right.clone();
        }
        // compute any new outputs & broadcast them.
        for autom in right.iter() {
            self.broadcast_partial(node, autom.apply_to_partial(partial));
        }
    }
    fn ynode_input_right(&mut self, node: &Rc<YNode>, autom: Automation) {
        let left;
        {
            let mut state = self.get_ynode_state(node);
            state.right.push(autom);
            // need to clone the State's left-hand-side to avoid mut issues
            left = state.left.clone();
        }
        // compute any new outputs & broadcast them.
        for partial in left.iter() {
            self.broadcast_partial(node, autom.apply_to_partial(*partial));
        }
    }
    fn anode_input(&mut self, node: &Rc<ANode>, slot: NodeInputSlot,
      autom: Automation) {
        match slot {
            NodeInputSlot::Left => {
                let right;
                {
                    let mut state = self.get_anode_state(node);
                    state.left.push(autom);
                    right = state.right.clone();
                }
                // all the automations in the right bin affect the new autom
                for autom_right in right.iter() {
                    self.broadcast_autom(node, autom_right.apply_to_autom(autom));
                }
            },
            NodeInputSlot::Right => {
                let left;
                {
                    let mut state = self.get_anode_state(node);
                    state.right.push(autom);
                    left = state.left.clone();
                }
                // this new autom affects all automations in the left bin
                for autom_left in left.iter() {
                    self.broadcast_autom(node, autom.apply_to_autom(*autom_left));
                }
            },
        };
    }
}

impl Tree for TreeRenderer {
    /// Connect two nodes in the tree.
    /// If the send is ASrcSend / YSrcSend, this is the same as feeding
    /// external stimuli into the tree.
    fn add_send(&mut self, send: Send) {
        match send {
            Send::AASend(send) => {
                self.get_anode_state(send.src()).add_aasend(send);
            },
            Send::AYSend(send) => {
                self.get_anode_state(send.src()).add_aysend(send);
            },
            Send::YYSend(send) => {
                self.get_ynode_state(send.src()).add_yysend(send);
            },
            Send::ASrcSend(send) => {
                self.broadcast_autom(send.dest(), *send.src());
            },
            Send::YSrcSend(send) => {
                self.broadcast_partial(send.dest(), *send.src());
            },
        }
    }

    /// set the nodes for which we are interested in the output PCM signals.
    /// Future calls to `step()` will return an array of samples corresponding
    /// to these nodes.
    fn watch_nodes(&mut self, outputs: &[Rc<YNode>]) {
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


impl ANodeState {
    fn new() -> ANodeState {
        ANodeState{ left:vec![], right: vec![], aasends: vec![], aysends:vec![] }
    }
    fn add_aasend(&mut self, send: AASend) {
        self.aasends.push(send);
    }
    fn add_aysend(&mut self, send: AYSend) {
        self.aysends.push(send);
    }
}

impl YNodeState {
    fn new() -> YNodeState {
        YNodeState{ left:vec![], right: vec![], yysends: vec![] }
    }
    fn add_yysend(&mut self, send: YYSend) {
        self.yysends.push(send);
    }
}

impl OutputState {
    fn new(spec: RenderSpec, node: Rc<YNode>) -> OutputState {
        OutputState{ node: node, renderer: PartialRenderer::new(spec) }
    }
    fn node(&self) -> &Rc<YNode> {
        &self.node
    }
    fn step(&mut self) -> f32 {
        self.renderer.step()
    }
    fn feed(&mut self, partial: Partial) {
        self.renderer.feed(partial);
    }
}

