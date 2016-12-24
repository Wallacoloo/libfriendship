extern crate online_dag;
extern crate pwline;
use self::online_dag::poscostdag;
use self::online_dag::poscostdag::{CostQueriable, PosCostDag};
use self::online_dag::ondag::OnDag;
pub use self::pwline::PwLine;

#[derive(PartialEq, Eq, Clone)]
pub struct RouteEdge {
    /// 0 corresponds to the source,
    /// 1 corresponds to the delay-by-zero weight,
    /// 2 corresponds to the delay-by-one weight, etc.
    slot_idx: u32,
}

pub enum LeafNode {
    /// Pointwise Lines usually used for automations, maybe impulses, etc.
    PwLine(PwLine<u32, f32>),
    /// retrieve a buffer of samples offset by the sample count of the first argument.
    FnPtr(Box<fn(u32, &mut [f32])>),
}

pub enum RouteNode {
    /// An intermediary node, which combines audio from upstream sources
    Intermediary,
    /// A leaf node, which generates audio on its own (i.e. spuriously).
    Leaf(LeafNode),
}

/// LeafNode get_samples function that fills a buffer with zeros.
/// Default implementation.
pub fn get_zeros(_start: u32, into: &mut [f32]) {
    for f in into.iter_mut() {
        *f = 0f32;
    }
}

impl LeafNode {
    /// fill the entire `into` buffer of samples based on external input at time t=offset.
    pub fn fill(&self, offset: u32, into: &mut [f32]) {
        match self {
            &LeafNode::PwLine(ref pwline) => {
                pwline.get_consecutive(offset, into);
            }
            &LeafNode::FnPtr(ref func) => {
                (func)(offset, into);
            }
        }
    }
}

//pub type RouteNodeHandle=<PosCostDag<RouteNode, RouteEdge> as OnDag<RouteNode, RouteEdge>>::NodeHandle;
// Prefer this syntax so we can have access to RouteNodeHandle::null(), etc.
pub type RouteNodeHandle=poscostdag::NodeHandle<RouteNode, RouteEdge>;
pub type WeakNodeHandle=poscostdag::WeakNodeHandle<RouteNode, RouteEdge>;
pub type FullEdge=poscostdag::FullEdge<RouteNode, RouteEdge>;
type DagImpl=PosCostDag<RouteNode, RouteEdge>;
pub struct RouteTree {
    dag: DagImpl,
    root: RouteNodeHandle,
}

impl RouteTree {
    pub fn new() -> Self {
        let mut s = RouteTree {
            dag: DagImpl::new(),
            root: RouteNodeHandle::null(),
        };
        s.root = s.dag.add_node(RouteNode::Intermediary);
        s
    }
    pub fn root(&self) -> &RouteNodeHandle {
        &self.root
    }
    pub fn iter_topo_rev(&self) -> impl Iterator<Item=poscostdag::NodeHandle<RouteNode, RouteEdge>> {
        self.dag.iter_topo_rev(&self.root)
    }
    pub fn children_of(&self, of: &RouteNodeHandle) -> impl Iterator<Item=poscostdag::HalfEdge<RouteNode, RouteEdge>> {
        self.dag.children(of)
    }
    pub fn add_node(&mut self, data: RouteNode) -> RouteNodeHandle {
        self.dag.add_node(data)
    }
    pub fn add_edge(&mut self, from: &RouteNodeHandle, to: &RouteNodeHandle, data: RouteEdge) -> Result<(), ()> {
        self.dag.add_edge(from, to, data)
    }
    pub fn rm_edge(&mut self, from: &RouteNodeHandle, to: &RouteNodeHandle, data: RouteEdge) {
        self.dag.rm_edge(from, to, data);
    }
    /*
    /// Return only the inputs into the left (i.e. non-delayed) channel of `of`
    pub fn left_children_of(&self, of: &RouteNodeHandle) -> impl Iterator<Item=poscostdag::HalfEdge<RouteNode, RouteEdge>> {
        self.dag.children(of).filter(|edge| edge.weight().is_left())
    }
    /// Return only the inputs into the right (i.e. non-delayed) channel of `of`
    pub fn right_children_of(&self, of: &RouteNodeHandle) -> impl Iterator<Item=poscostdag::HalfEdge<RouteNode, RouteEdge>> {
        self.dag.children(of).filter(|edge| edge.weight().is_right())
    }*/
}

impl RouteEdge {
    pub fn new_left() -> Self {
        RouteEdge{ slot_idx: 0 }
    }
    pub fn new_right(slot_idx: u32) -> Self {
        RouteEdge{ slot_idx: 1+slot_idx }
    }
    pub fn slot_idx(&self) -> usize {
        self.slot_idx as usize
    }
    pub fn is_left(&self) -> bool {
        self.slot_idx == 0
    }
    pub fn is_right(&self) -> bool {
        !self.is_left()
    }
    /// Returns the amount a right input is delayed,
    /// or 0 if the input is to the left slot
    pub fn delay(&self) -> u32 {
        if self.is_left() {
            0
        } else {
            self.slot_idx - 1
        }
    }
}

// default is needed for RouteNodeHandle::null
impl Default for RouteNode {
    fn default() -> Self {
        RouteNode::Intermediary
    }
}

// CostQueriable is needed for cycle prevention
impl CostQueriable<RouteNode, RouteEdge> for RouteEdge {
    fn is_zero_cost(my_edge: &FullEdge, dag : &DagImpl) -> bool {
        // Cost represents the delay of this data going into the next node.
        // If this is a right edge, delay is encoded in the edge (assuming there is SOME left
        // input)
        // If this is a left edge, delay is the minimum delay of all right nodes entering the
        // same node.
        if my_edge.weight().is_right() {
            dag.children(my_edge.to()).any(|in_edge| {
                in_edge.weight().is_left()
            })
        } else {
            dag.children(my_edge.to()).any(|in_edge| {
                in_edge.weight().is_right() && in_edge.weight().delay() == 0
            })
        }
    }
}
