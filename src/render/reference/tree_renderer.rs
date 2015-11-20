use std::collections::hash_map::HashMap;
use std::rc::Rc;

use automation::Automation;
use partial::Partial;
use tree::node::{ANode, YNode};
use tree::send::{AASend, AYSend, Send, YYSend};
use tree::tree::Tree;

/// Tracks all the partials/automations that have ever entered the node.
/// This information is needed in order to evaluate the output whenever
/// a new input arrives.
struct ANodeState {
    left: Vec<Automation>,
    right: Vec<Automation>,
}

/// Tracks all the partials/automations that have ever entered the node.
/// This information is needed in order to evaluate the output whenever
/// a new input arrives.
struct YNodeState {
    left: Vec<Partial>,
    right: Vec<Automation>,
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
    fn get_anode_state(&mut self, node: &Rc<ANode>) -> &mut ANodeState {
        unimplemented!();
    }
    fn get_ynode_state(&mut self, node: &Rc<YNode>) -> &mut YNodeState {
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
                self.get_anode_state(send.dest()).add_output(send.src());
            },
            Send::YSrcSend(send) => {
                self.get_ynode_state(send.dest()).add_output(send.src());
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
    fn add_output(&mut self, out: &Automation) {
        unimplemented!();
    }
}

impl YNodeState {
    fn add_yysend(&mut self, send: YYSend) {
        unimplemented!();
    }
    fn add_output(&mut self, out: &Partial) {
        unimplemented!();
    }
}