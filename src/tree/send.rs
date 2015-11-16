use std::rc::Rc;

use super::node::{ANode, NodeInputSlot, YNode};

/// Sends an automation stream from the output of an ANode to the input of
/// another ANode (either the left or right slot, as specified)
pub struct AASend {
    src: Rc<ANode>,
    dest: Rc<ANode>,
    dest_slot: NodeInputSlot,
}

/// Sends an automation stream from the output of a ANode to the (right) input
/// of a YNode
pub struct AYSend {
    src: Rc<ANode>,
    dest: Rc<YNode>,
}

/// Sends a partial stream from the output of a YNode to the (left) input of
///   another YNode.
pub struct YYSend {
    src: Rc<YNode>,
    dest: Rc<YNode>,
}

pub enum Send {
    AASend(AASend),
    AYSend(AYSend),
    YYSend(YYSend),
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

