use std::cmp::{Ord, Ordering};
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
pub struct ANode {
    id: NodeId,
}

/// Takes two inputs:
///   Y (a sum of partials)
///   A (a sum of automations)
/// Provides one output: Y*A, which is a sum of partials
pub struct YNode {
    id: NodeId,
}

pub enum Node {
    ANode(ANode),
    YNode(YNode),
}

/// Used primarily in Sends; Automations can either be sent to the *left* input
/// of an ANode, or to the right input.
#[derive(Clone, Copy, Debug)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
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
    fn id(&self) -> NodeId {
        match self {
            &Node::ANode(ref anode) => anode.id(),
            &Node::YNode(ref ynode) => ynode.id(),
        }
    }
}

impl ANode {
    pub fn new() -> ANode {
        ANode{ id: Node::consume_next_id() }
    }
    fn id(&self) -> NodeId {
        self.id
    }
}

impl YNode {
    pub fn new() -> YNode {
        YNode{ id: Node::consume_next_id() }
    }
    fn id(&self) -> NodeId {
        self.id
    }
}


// Implement comparisons between Nodes so that they can be orderable.
impl Ord for Node {
    fn cmp(&self, other: &Node) -> Ordering {
        self.id().cmp(&other.id())
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Node) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl Eq for Node {}


// Comparisons for ANode
impl Ord for ANode {
    fn cmp(&self, other: &ANode) -> Ordering {
        self.id().cmp(&other.id())
    }
}

impl PartialOrd for ANode {
    fn partial_cmp(&self, other: &ANode) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ANode {
    fn eq(&self, other: &ANode) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl Eq for ANode {}


// Comparisons for YNode
impl Ord for YNode {
    fn cmp(&self, other: &YNode) -> Ordering {
        self.id().cmp(&other.id())
    }
}

impl PartialOrd for YNode {
    fn partial_cmp(&self, other: &YNode) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for YNode {
    fn eq(&self, other: &YNode) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl Eq for YNode {}