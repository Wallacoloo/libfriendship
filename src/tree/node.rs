use std::rc::Rc;
use std::sync::atomic;
use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT};

/// Each node is given a unique ID.
/// This is necessary because these nodes need to comparable,
/// and pointer comparison might not be valid if the type has zero size and is
/// stored in an optimized container.
type NodeId = u32;

#[derive(Debug)]
#[derive(PartialEq, Eq)]
#[derive(Hash)]
pub enum NodeOp {
    // Evaluates y1 AT y2. i.e. y1 is delayed by y2's modulation parameter and
    // then multiplied by y2.
    OpAt,
    // Straight-up arithmetic multiplication, but the new modulation parameter
    // is (a1-a2)
    OpBy,
}

/// Takes two inputs:
///   Y1 (a sum of signals)
///   Y2 (a sum of signals)
/// Provides one output: Y1 <op> Y2, which is a sum of signals
#[derive(Debug)]
#[derive(PartialEq, Eq)]
#[derive(Hash)]
pub struct Node {
    id: NodeId,
    op: NodeOp,
}

/// Used primarily in Sends; Signals can either be sent to the *left* input
/// of a Node, or to the right input.
#[derive(Clone, Copy, Debug)]
#[derive(PartialEq, Eq)]
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
    pub fn new(op : NodeOp) -> Node {
        Node{ id: Node::consume_next_id(), op: op }
    }
    pub fn new_rc(op : NodeOp) -> Rc<Node> {
        Rc::new(Node::new(op))
    }
    pub fn op(&self) -> &NodeOp {
        &self.op
    }
}

