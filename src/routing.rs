extern crate online_dag;
extern crate pwline;
use self::online_dag::rcdag;
use self::online_dag::rcdag::RcDag;
use self::online_dag::ondag::OnDag;
use self::pwline::PwLine;

#[derive(PartialEq, Eq, Clone)]
pub struct RouteEdge {
    /// 0 corresponds to the source,
    /// 1 corresponds to the delay-by-zero weight,
    /// 2 corresponds to the delay-by-one weight, etc.
    slot_idx: u32,
}

pub enum LeafNode {
    PwLine(PwLine<u32, f32>),
    /// retrieve a buffer of samples offset by the sample count of the first argument.
    FnPtr(Box<fn(u32, &mut [f32])>),
}

pub enum RouteNode {
    /// An intermediary node, which combines audio from upstream sources
    Intermediary,
    /// A leaf node, which generates audio on its own.
    Leaf(LeafNode),
}

/// LeafNode get_samples function that fills a buffer with zeros.
/// Default implementation.
pub fn get_zeros(start: u32, into: &mut [f32]) {
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

pub type RouteNodeHandle=<RcDag<RouteNode, RouteEdge> as OnDag<RouteNode, RouteEdge>>::NodeHandle;
pub struct RouteTree {
    dag: RcDag<RouteNode, RouteEdge>,
    root: RouteNodeHandle,
}

impl RouteTree {
    pub fn iter_topo_rev(&self) -> impl Iterator<Item=rcdag::NodeHandle<RouteNode, RouteEdge>> {
        self.dag.iter_topo_rev(&self.root)
    }
}
