use std::rc::Rc;

use signal::Signal;

use super::node::{Node, NodeInputSlot};

/// Sends a Signal stream from the output of one Node to the input of
/// another Node (either the left or right slot, as specified)
#[derive(Clone, Debug)]
#[derive(PartialEq, Eq)]
#[derive(Hash)]
pub struct NodeSend {
    src: Rc<Node>,
    dest: Rc<Node>,
    dest_slot: NodeInputSlot,
}

/// Sends a Signal stream from an external stimuli into the output slot of
/// the Node
#[derive(Clone, Debug)]
#[derive(PartialEq, Eq)]
#[derive(Hash)]
pub struct SrcSend {
    src: Signal,
    dest: Rc<Node>,
}

#[derive(Clone, Debug)]
#[derive(PartialEq, Eq)]
#[derive(Hash)]
pub enum Send {
    NodeSend(NodeSend),
    SrcSend(SrcSend),
}

impl Send {
    /// Convenience function to create a NodeSend in tagged enum form
    pub fn new_nodesend(src: Rc<Node>, dest: Rc<Node>, dest_slot: NodeInputSlot)
    -> Send {
        Send::NodeSend(NodeSend::new(src, dest, dest_slot))
    }
    /// Convenience function to create a SrcSend in tagged enum form
    pub fn new_srcsend(src: Signal, dest: Rc<Node>) -> Send {
        Send::SrcSend(SrcSend::new(src, dest))
    }
}

impl NodeSend {
    pub fn new(src: Rc<Node>, dest: Rc<Node>, dest_slot: NodeInputSlot)
    -> NodeSend {
        NodeSend{ src: src, dest:dest, dest_slot: dest_slot }
    }
    pub fn src(&self) -> &Rc<Node> {
        &self.src
    }
    pub fn dest(&self) -> &Rc<Node> {
        &self.dest
    }
    pub fn dest_slot(&self) -> NodeInputSlot {
        self.dest_slot
    }
}


impl SrcSend {
    pub fn new(src: Signal, dest: Rc<Node>)
    -> SrcSend {
        SrcSend{ src: src, dest:dest }
    }
    pub fn src(&self) -> &Signal {
        &self.src
    }
    pub fn dest(&self) -> &Rc<Node> {
        &self.dest
    }
}

