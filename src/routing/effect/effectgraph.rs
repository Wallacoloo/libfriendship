extern crate online_dag;

use self::online_dag::poscostdag;
use self::online_dag::poscostdag::{CostQueriable, PosCostDag};
use self::online_dag::ondag::OnDag;
use super::effect::Effect;

#[derive(Clone, PartialEq, Eq)]
struct EffectEdge {
    /// The source index is the index of the *output* from the source node
    source_idx: u32,
    /// The source index is the index to the *input* of the next node.
    dest_idx: u32,
}

pub type EffectNodeHandle=poscostdag::NodeHandle<Effect, EffectEdge>;
pub type EffectHalfEdge=poscostdag::HalfEdge<Effect, EffectEdge>;
type DagImpl=PosCostDag<Effect, EffectEdge>;

pub struct EffectGraph {
    dag: DagImpl,
    root: EffectNodeHandle,
}

impl EffectGraph {
    pub fn new(n_channels: u32) -> Self {
        let mut s = EffectGraph {
            dag: DagImpl::new(),
            root: EffectNodeHandle::null(),
        };
        s.root = s.dag.add_node(Effect::new("passthrough".to_string(), n_channels));
        s
    }
}

// CostQueriable is needed for cycle prevention
impl CostQueriable<Effect, EffectEdge> for EffectEdge {
    fn is_zero_cost(my_edge: &EffectHalfEdge, _next_edge: &EffectHalfEdge, dag : &DagImpl) -> bool {
        // In our DAG, the edges have zero delay and the nodes present a delay from their input to
        // their output.
        let source_idx = my_edge.weight().source_idx;
        let dest_idx = my_edge.weight().dest_idx;
        my_edge.to().node_data().min_latency(source_idx, dest_idx) == Some(0)
    }
}
