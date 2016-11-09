extern crate online_dag;
use self::online_dag::rcdag::RcDag;

pub struct RouteEdge {
    /// 0 corresponds to the source,
    /// 1 corresponds to the delay-by-zero weight,
    /// 2 corresponds to the delay-by-one weight, etc.
    slot_idx: u32,
}

pub struct LeafNode {
    /// retrieve a buffer of samples offset by the sample count of the first argument.
    get_samples: Box<fn(u32, &mut [f32])>,
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
    pub fn new(get_samples: Box<fn(u32, &mut [f32])>) -> Self {
        LeafNode{
            get_samples: get_samples,
        }
    }
    /// fill the entire `into` buffer of samples based on external input at time t=offset.
    pub fn fill(&self, offset: u32, into: &mut [f32]) {
        (self.get_samples)(offset, into);
    }
}

pub type RouteTree=RcDag<RouteNode, RouteEdge>;
