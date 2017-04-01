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
use std::collections::hash_map::HashMap;

type DagImpl=IODag<NodeData, EdgeWeight>;
type PrimNodeHandle=iodag::NodeHandle;
type PrimEdge=iodag::Edge<EdgeWeight>;

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
struct EdgeWeight {
    from_slot: u32,
    from_ch: u8,
    to_slot: u32,
    to_ch: u8,
}

enum NodeData {
    Effect(Effect),
    Graph(DagHandle),
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct DagHandle {
    id: u32,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct NodeHandle {
    dag_handle: DagHandle,
    node_handle: PrimNodeHandle,
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct Edge {
    dag_handle: DagHandle,
    edge: PrimEdge,
}


pub struct RouteGraph {
    dag_counter: u32,
    dags: HashMap<DagHandle, DagImpl>,
    watchers: Vec<Box<GraphWatcher>>,
}

impl RouteGraph {
    pub fn new() -> Self {
        RouteGraph {
            dag_counter: 0,
            dags: HashMap::new(),
            watchers: Vec::new(),
        }
    }
    pub fn add_watcher(&mut self, mut watcher: Box<GraphWatcher>, do_replay: bool) {
        if do_replay {
            for (dag_handle, dag) in self.dags.iter() {
                for node in dag.iter_nodes() {
                    watcher.on_add_node(&NodeHandle::new(*dag_handle, *node));
                }
                for edge in dag.iter_edges() {
                    watcher.on_add_edge(&Edge::new(*dag_handle, edge.clone()));
                }
            }
        }
        self.watchers.push(watcher);
    }
    pub fn add_dag(&mut self) -> DagHandle {
        let handle = DagHandle{ id: self.dag_counter};
        self.dag_counter += 1;
        self.dags.insert(handle, DagImpl::new());
        handle
    }
    pub fn add_node(&mut self, dag: DagHandle, node: NodeData) -> NodeHandle {
        let inner_handle = self.dags.get_mut(&dag).unwrap().add_node(node);
        let handle = NodeHandle::new(dag, inner_handle);
        for w in &mut self.watchers {
            w.on_add_node(&handle);
        }
        handle
    }
    pub fn add_edge(&mut self, edge: Edge) -> Result<(), ()> {
        let ok_to_add = {
            let ref dag = self.dags[edge.dag_handle()];
            dag.can_add_edge(edge.edge(),
                &|e1, e2| self.are_slots_connected(
                    dag.node_data(e1.to().unwrap()),
                    e1.weight().to_slot, e1.weight().to_ch,
                    e2.weight().from_slot, e2.weight().from_ch
                )
            )
        };
        if let Ok(_) = ok_to_add {
            // only notify watchers on a successful operation.
            for w in &mut self.watchers {
                w.on_add_edge(&edge);
            }
            let mut mdag = self.dags.get_mut(edge.dag_handle()).unwrap();
            mdag.add_edge_unchecked(edge.edge);
        }
        ok_to_add
    }
    fn are_slots_connected(&self, data: &NodeData, in_slot: u32, in_ch: u8, out_slot: u32, out_ch: u8) -> bool {
        match *data {
            // TODO: for now, consider all inputs tied to all outputs for each graph.
            // In future, may enforce constraints or actually calculate the connections,
            // but this requires careful planning due to aliasing.
            NodeData::Graph(ref dag_handle) => true,
            NodeData::Effect(ref effect) => effect.are_slots_connected(in_slot, in_ch, out_slot, out_ch)
        }
    }
    pub fn del_node(&mut self, node: NodeHandle) -> Result<(), ()> {
        let result = self.dags.get_mut(node.dag_handle()).unwrap().del_node(*node.node_handle());
        // only notify watchers on a successful operation.
        if let Ok(_) = result {
            for w in &mut self.watchers {
                w.on_del_node(&node);
            }
        }
        result
    }
    pub fn del_edge(&mut self, edge: Edge) {
        self.dags.get_mut(edge.dag_handle()).unwrap().del_edge(edge.edge().clone());
        for w in &mut self.watchers {
            w.on_del_edge(&edge);
        }
    }
}

impl NodeHandle {
    fn new(dag: DagHandle, node: PrimNodeHandle) -> Self {
        Self {
            dag_handle: dag,
            node_handle: node,
        }
    }
    fn dag_handle(&self) -> &DagHandle {
        &self.dag_handle
    }
    fn node_handle(&self) -> &PrimNodeHandle {
        &self.node_handle
    }
}

impl Edge {
    fn new(dag: DagHandle, edge: PrimEdge) -> Self {
        Self {
            dag_handle: dag,
            edge: edge,
        }
    }
    fn dag_handle(&self) -> &DagHandle {
        &self.dag_handle
    }
    fn edge(&self) -> &PrimEdge {
        &self.edge
    }
}

