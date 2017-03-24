/// RouteGraph defines a Directed Acyclic Graph of Effects.
/// The edges connecting each Effect have a source and destination slot, tag, and channel.
/// Edges are also allowed to go to null, in which case they only have a destination slot and
/// channel. These are outputs.
/// Edges can also COME from null, in which case the source has the format (slot, channel)
extern crate online_dag;
use self::online_dag::iodag;
use self::online_dag::iodag::IODag;

use super::effect::Effect;

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


type DagImpl=IODag<NodeData, EdgeWeight>;
pub type NodeHandle=iodag::NodeHandle;
pub type Edge=iodag::Edge<EdgeWeight>;
pub struct RouteGraph {
    dag: DagImpl,
}

impl RouteGraph {
    pub fn new() -> Self {
        RouteGraph {
            dag: DagImpl::new(),
        }
    }
    /*pub fn iter_topo_rev(&self) -> impl Iterator<Item=poscostdag::NodeHandle<RouteNode, RouteEdge>> {
        self.dag.iter_topo_rev(&self.root)
    }*/
    /*pub fn children_of(&self, of: &RouteNodeHandle) -> impl Iterator<Item=RouteHalfEdge> {
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
    /// Return only the inputs into the left (i.e. non-delayed) channel of `of`
    pub fn left_children_of(&self, of: &RouteNodeHandle) -> impl
      Iterator<Item=RouteHalfEdge> {
        self.dag.children(of).filter(|edge| edge.weight().is_left())
    }
    /// Return only the inputs into the right (i.e. non-delayed) channel of `of`
    pub fn right_children_of(&self, of: &RouteNodeHandle) -> impl
      Iterator<Item=RouteHalfEdge> {
        self.dag.children(of).filter(|edge| edge.weight().is_right())
    }
    /// Returns 1 + the index of the highest channel number.
    /// e.g. if we have ch0 and ch2 (no ch1), this returns 3.
    pub fn n_channels(&self) -> u32 {
        self.right_children_of(&self.root())
            .map(|edge| edge.weight().slot_idx())
            .max()
            .unwrap_or(0u32) // default no channels
    }
    pub fn make_channel_output(&mut self, node: &RouteNodeHandle, ch: u32) {
        let root = &self.root.clone();
        self.add_edge(&root, &node, RouteEdge::new_right(ch));
    }*/
}
