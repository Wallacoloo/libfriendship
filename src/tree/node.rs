use std::cmp::{Ord, Ordering};
use std::hash::Hash;
use std::sync::atomic;
use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT};

/// Each node is given a unique ID.
/// This is necessary because these nodes need to comparable,
/// and pointer comparison might not be valid if the type has zero size and is
/// stored in an optimized container.
type NodeId = u32;

/// Takes two inputs:
///   A1 (a sum of automations)
///   A2 (a sum of automations)
/// Provides one output: A1*A2, which is a sum of automations
#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[derive(Hash)]
pub struct ANode {
    id: NodeId,
}

/// Takes two inputs:
///   Y (a sum of partials)
///   A (a sum of automations)
/// Provides one output: Y*A, which is a sum of partials
#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[derive(Hash)]
pub struct YNode {
    id: NodeId,
}

/// Node that spontaneously generates Automations
#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[derive(Hash)]
pub struct ASource {
    id: NodeId,
}

/// Node that spontaneously generates Partials
#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[derive(Hash)]
pub struct YSource {
    id: NodeId,
}

/// Node whose output may be intercepted and routed out of the tree.
    /// Typically, there is one of these for each audio channel.
    /// (e.g. two for a stereo system)#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[derive(Hash)]
pub struct YSink {
    id: NodeId,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[derive(Hash)]
pub enum Node {
    ANode(ANode),
    YNode(YNode),
    ASource(ASource),
    YSource(YSource),
    YSink(YSink),
}

/// Used primarily in Sends; Automations can either be sent to the *left* input
/// of an ANode, or to the right input.
#[derive(Clone, Copy, Debug)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[derive(Hash)]
pub enum NodeInputSlot {
    Left,
    Right,
}



static mut prev_node_id : AtomicUsize = ATOMIC_USIZE_INIT;

impl Node {
    fn consume_next_id() -> NodeId {
        let id = unsafe {
            prev_node_id.fetch_add(1, atomic::Ordering::SeqCst)
        };
        id as u32
    }
}

impl ANode {
    pub fn new() -> ANode {
        ANode{ id: Node::consume_next_id() }
    }
}

impl YNode {
    pub fn new() -> YNode {
        YNode{ id: Node::consume_next_id() }
    }
}

impl ASource {
    pub fn new() -> ASource {
        ASource{ id: Node::consume_next_id() }
    }
}

impl YSource {
    pub fn new() -> YSource {
        YSource{ id: Node::consume_next_id() }
    }
}

impl YSink {
    pub fn new() -> YSink {
        YSink{ id: Node::consume_next_id() }
    }
}
