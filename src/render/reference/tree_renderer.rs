use std::collections::hash_map::HashMap;
use std::rc::Rc;

use automation::Automation;
use partial::Partial;
use tree::node::{ANode, NodeInputSlot, YNode};
use tree::send::{AASend, AYSend, Send, YYSend};
use tree::tree::Tree;

/// Tracks all the partials/automations that have ever entered the node.
/// This information is needed in order to evaluate the output whenever
/// a new input arrives.
struct ANodeState {
    left: Vec<Automation>,
    right: Vec<Automation>,
    aasends: Vec<AASend>,
    aysends: Vec<AYSend>
}

/// Tracks all the partials/automations that have ever entered the node.
/// This information is needed in order to evaluate the output whenever
/// a new input arrives.
struct YNodeState {
    left: Vec<Partial>,
    right: Vec<Automation>,
    yysends: Vec<YYSend>,
}


struct TreeRenderer {
    anode_states: HashMap<Rc<ANode>, ANodeState>,
    ynode_states: HashMap<Rc<YNode>, YNodeState>,
    output_nodes: Vec<Rc<YNode>>,
    output_buff: Vec<f32>,
}

impl TreeRenderer {
    pub fn new() -> TreeRenderer {
        TreeRenderer{
            anode_states: HashMap::new(),
            ynode_states: HashMap::new(),
            output_nodes: vec![],
            output_buff: vec![]
        }
    }
    /// Get the ANodeState associated with a node,
    /// and create it if it doesn't already exist.
    fn get_anode_state(&mut self, node: &Rc<ANode>) -> &mut ANodeState {
        unimplemented!();
        //&self.anode_states[node]
    }
    /// Get the YNodeState associated with a node,
    /// and create it if it doesn't already exist.
    fn get_ynode_state(&mut self, node: &Rc<YNode>) -> &mut YNodeState {
        unimplemented!();
        //&self.ynode_states[node]
    }
    /// Send a partial to the input of any nodes connected to the output of the
    /// given node.
    fn broadcast_partial(&mut self, from: &Rc<YNode>, partial: Partial) {
        // need to clone the State's sends to avoid mutability problems
        let sends = self.get_ynode_state(from).yysends.clone();
        for ref send in sends.iter() {
            self.ynode_input_left(send.dest(), partial);
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
        unimplemented!();
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
        self.output_nodes = outputs.to_vec();
    }
    /// Return the next buffer of samples related to the watched nodes.
    fn step(&mut self) -> &[f32] {
        unimplemented!();
        &self.output_buff
    }
}


impl ANodeState {
    fn add_aasend(&mut self, send: AASend) {
        unimplemented!();
    }
    fn add_aysend(&mut self, send: AYSend) {
        unimplemented!();
    }
}

impl YNodeState {
    fn add_yysend(&mut self, send: YYSend) {
        unimplemented!();
    }
}