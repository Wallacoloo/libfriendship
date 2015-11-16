use std::cmp::{Ord, Ordering, PartialEq, PartialOrd};
use std::hash::Hash;
use std::rc::Rc;

use super::node::{ANode, ASrcNode, Node, NodeInputSlot, YNode, YSinkNode,
    YSrcNode};

/// Sends an Automation stream from the output of an ANode to the input of
/// another ANode (either the left or right slot, as specified)
#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[derive(Hash)]
pub struct AASend {
    src: Rc<ANode>,
    dest: Rc<ANode>,
    dest_slot: NodeInputSlot,
}

/// Sends an Automation stream from the output of a ANode to the (right) input
/// of a YNode
#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[derive(Hash)]
pub struct AYSend {
    src: Rc<ANode>,
    dest: Rc<YNode>,
}

/// Sends a Partial stream from the output of a YNode to the (left) input of
///   another YNode.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[derive(Hash)]
pub struct YYSend {
    src: Rc<YNode>,
    dest: Rc<YNode>,
}

/// Sends an Automation stream from an external stimuli into the ANode
#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[derive(Hash)]
pub struct ASrcSend {
    src: Rc<ASrcNode>,
    dest: Rc<ANode>,
    dest_slot: NodeInputSlot,
}

/// Sends a Partial stream from an external stimuli into the YNode
#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[derive(Hash)]
pub struct YSrcSend {
    src: Rc<YSrcNode>,
    dest: Rc<YNode>,
}

/// Sends a Partial stream to a node that may be connected to the outside world.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[derive(Hash)]
pub struct YSinkSend {
    src: Rc<YNode>,
    dest: Rc<YSinkNode>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[derive(Hash)]
pub enum Send {
    AASend(AASend),
    AYSend(AYSend),
    YYSend(YYSend),
    ASrcSend(ASrcSend),
    YSrcSend(YSrcSend),
    YSinkSend(YSinkSend),
}

impl AASend {
    pub fn new(src: Rc<ANode>, dest: Rc<ANode>, dest_slot: NodeInputSlot)
    -> AASend {
        AASend{ src: src, dest:dest, dest_slot: dest_slot }
    }
    pub fn src(&self) -> &Rc<ANode> {
        &self.src
    }
    pub fn dest(&self) -> &Rc<ANode> {
        &self.dest
    }
    pub fn dest_slot(&self) -> NodeInputSlot {
        self.dest_slot
    }
}

impl AYSend {
    pub fn new(src: Rc<ANode>, dest: Rc<YNode>) -> AYSend {
        AYSend{ src: src, dest:dest }
    }
    pub fn src(&self) -> &Rc<ANode> {
        &self.src
    }
    pub fn dest(&self) -> &Rc<YNode> {
        &self.dest
    }
}

impl YYSend {
    pub fn new(src: Rc<YNode>, dest: Rc<YNode>) -> YYSend {
        YYSend{ src: src, dest:dest }
    }
    pub fn src(&self) -> &Rc<YNode> {
        &self.src
    }
    pub fn dest(&self) -> &Rc<YNode> {
        &self.dest
    }
}


impl ASrcSend {
    pub fn new(src: Rc<ASrcNode>, dest: Rc<ANode>, dest_slot: NodeInputSlot)
    -> ASrcSend {
        ASrcSend{ src: src, dest:dest, dest_slot: dest_slot }
    }
    pub fn src(&self) -> &Rc<ASrcNode> {
        &self.src
    }
    pub fn dest(&self) -> &Rc<ANode> {
        &self.dest
    }
    pub fn dest_slot(&self) -> NodeInputSlot {
        self.dest_slot
    }
}

impl YSrcSend {
    pub fn new(src: Rc<YSrcNode>, dest: Rc<YNode>) -> YSrcSend {
        YSrcSend{ src: src, dest:dest }
    }
    pub fn src(&self) -> &Rc<YSrcNode> {
        &self.src
    }
    pub fn dest(&self) -> &Rc<YNode> {
        &self.dest
    }
}

impl YSinkSend {
    pub fn new(src: Rc<YNode>, dest: Rc<YSinkNode>) -> YSinkSend {
        YSinkSend{ src: src, dest:dest }
    }
    pub fn src(&self) -> &Rc<YNode> {
        &self.src
    }
    pub fn dest(&self) -> &Rc<YSinkNode> {
        &self.dest
    }
}