/// RouteGraph defines a Directed Acyclic Graph of Effects.
/// The edges connecting each Effect have a source and destination slot, tag, and channel.
/// Edges are also allowed to go to null, in which case they only have a destination slot and
/// channel. These are outputs.
/// Edges can also COME from null, in which case the source has the format (slot, channel)
extern crate online_dag;
use self::online_dag::iodag;
use self::online_dag::iodag::IODag;

use super::effect::Effect;

type DagImpl=IODag<NodeData, EdgeWeight>;
pub type NodeHandle=iodag::NodeHandle;
pub type Edge=iodag::Edge<EdgeWeight>;

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
struct EdgeWeight {
    from_slot: u32,
    from_ch: u8,
    to_slot: u32,
    to_ch: u8,
}

struct NodeData {
    effect: Effect,
}


pub struct RouteGraph {
    dag: DagImpl,
}

impl RouteGraph {
    pub fn new() -> Self {
        RouteGraph {
            dag: DagImpl::new(),
        }
    }
    pub fn add_node(&mut self, node: NodeData) -> NodeHandle {
        self.dag.add_node(node)
    }
    pub fn add_edge(&mut self, edge: Edge) -> Result<(), ()> {
        self.dag.add_edge(edge)
    }
    pub fn del_edge(&mut self, edge: Edge) {
        self.dag.del_edge(edge)
    }
}
