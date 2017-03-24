/// RouteGraph defines a Directed Acyclic Graph of Effects.
/// The edges connecting each Effect have a source and destination slot, tag, and channel.
/// Edges are also allowed to go to null, in which case they only have a destination slot and
/// channel. These are outputs.
/// Edges can also COME from null, in which case the source has the format (slot, channel)
extern crate online_dag;
use self::online_dag::iodag;
use self::online_dag::iodag::IODag;

use super::effect::Effect;
use super::graphwatcher::GraphWatcher;

use std::collections::hash_set::HashSet;

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
    watchers: Vec<Box<GraphWatcher>>,
}

impl RouteGraph {
    pub fn new() -> Self {
        RouteGraph {
            dag: DagImpl::new(),
            watchers: Vec::new(),
        }
    }
    pub fn add_watcher(&mut self, mut watcher: Box<GraphWatcher>, do_replay: bool) {
        if do_replay {
            for node in self.dag.iter_nodes() {
                watcher.on_add_node(&node);
            }
            for edge in self.dag.iter_edges() {
                watcher.on_add_edge(&edge);
            }
        }
        self.watchers.push(watcher);
    }
    pub fn add_node(&mut self, node: NodeData) -> NodeHandle {
        let handle = self.dag.add_node(node);
        for w in &mut self.watchers {
            w.on_add_node(&handle);
        }
        handle
    }
    pub fn add_edge(&mut self, edge: Edge) -> Result<(), ()> {
        let result = self.dag.add_edge(edge.clone());
        // only notify watchers on a successful operation.
        if let Ok(_) = result {
            for w in &mut self.watchers {
                w.on_add_edge(&edge);
            }
        }
        result
    }
    pub fn del_node(&mut self, node: NodeHandle) -> Result<(), ()> {
        let result = self.dag.del_node(node);
        // only notify watchers on a successful operation.
        if let Ok(_) = result {
            for w in &mut self.watchers {
                w.on_del_node(&node);
            }
        }
        result
    }
    pub fn del_edge(&mut self, edge: Edge) {
        for w in &mut self.watchers {
            w.on_del_edge(&edge);
        }
        self.dag.del_edge(edge)
    }
}
