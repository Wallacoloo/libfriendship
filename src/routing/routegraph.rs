/// RouteGraph defines a Directed Acyclic Graph of Effects.
/// The edges connecting each Effect have a source and destination slot, tag, and channel.
/// Edges are also allowed to go to null, in which case they only have a destination slot and
/// channel. These are outputs.
/// Edges can also COME from null, in which case the source has the format (slot, channel)

use super::effect::Effect;
use super::graphwatcher::GraphWatcher;

use std::collections::hash_set::HashSet;
use std::collections::hash_map::HashMap;
use std::collections::hash_map;
use std::rc::Rc;

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
struct EdgeWeight {
    from_slot: u32,
    from_ch: u8,
    to_slot: u32,
    to_ch: u8,
}

#[derive(Eq, PartialEq)]
enum NodeData {
    Effect(Rc<Effect>),
    Graph(DagHandle),
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct DagHandle {
    id: u32,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct PrimNodeHandle {
    id: u64,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct NodeHandle {
    dag_handle: DagHandle,
    node_handle: Option<PrimNodeHandle>,
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct Edge {
    dag_handle: DagHandle,
    from: Option<PrimNodeHandle>,
    to: Option<PrimNodeHandle>,
    weight: EdgeWeight,
}


pub struct RouteGraph {
    // TODO: can make these non-zero for more efficient Option<NodeHandle> encoding
    dag_counter: u32,
    node_counter: u64,
    watchers: Vec<Box<GraphWatcher>>,
    edges: HashMap<NodeHandle, EdgeSet>,
    node_data: HashMap<NodeHandle, NodeData>,
}

struct EdgeSet {
    outbound: HashSet<Edge>,
    inbound: HashSet<Edge>,
}

impl RouteGraph {
    pub fn new() -> Self {
        RouteGraph {
            dag_counter: 0,
            node_counter: 0,
            watchers: Vec::new(),
            edges: HashMap::new(),
            node_data: HashMap::new(),
        }
    }
    pub fn add_watcher(&mut self, mut watcher: Box<GraphWatcher>, do_replay: bool) {
        if do_replay {
            for node in self.iter_nodes() {
                watcher.on_add_node(node);
            }
            for edge in self.iter_edges() {
                watcher.on_add_edge(edge);
            }
        }
        self.watchers.push(watcher);
    }
    pub fn iter_nodes<'a>(&'a self) -> impl Iterator<Item=&NodeHandle> + 'a {
        self.node_data.keys()
    }
    pub fn iter_edges<'a>(&'a self) -> impl Iterator<Item=&Edge> + 'a {
        self.edges.values().flat_map(|v_set| v_set.outbound.iter())
    }
    pub fn add_dag(&mut self) -> DagHandle {
        let handle = DagHandle{ id: self.dag_counter };
        self.dag_counter += 1;
        handle
    }
    pub fn add_node(&mut self, dag: DagHandle, node_data: NodeData) -> NodeHandle {
        let primhandle = PrimNodeHandle { id: self.node_counter };
        let handle = NodeHandle {
            dag_handle: dag,
            node_handle: Some(primhandle),
        };
        self.node_counter = self.node_counter+1;
        // Create storage for the node's outgoing edges
        // Panic if the NodeHandle was somehow already in use.
        assert!(self.edges.insert(handle, EdgeSet::new()).is_none());
        // Store the node's data
        assert!(self.node_data.insert(handle, node_data).is_none());
        for w in &mut self.watchers {
            w.on_add_node(&handle);
        }
        handle
    }
    pub fn add_edge(&mut self, edge: Edge) -> Result<(), ()> {
        // Algorithm:
        //   Try to reach `edge` from `edge`.
        //   If we reach the boundary of the DAG while doing so, consider all reachable outbound
        //     edges of the DAG
        //     For each such edge, try to reach this DAG (recursively), and then resume the search for `edge`.
        let is_reachable = self.is_edge_reachable(&edge, &edge);
        // Locate all nodes that alias to this DAG:
        /*let aliased_nodes = self.edges.iter().filter(|&(k, v)| {
            k.dag_handle == edge.dag_handle
        });
        let edges_to_check = aliased_nodes.flat_map(|(k, v)| {
            v.outbound.iter()
        });*/
        // If a cycle was introduced (
        unimplemented!();
        /*let ok_to_add = {
            let ref dag = self.dags[edge.dag_handle()];
            dag.can_add_edge(edge.edge(),
                &|e1, e2| self.are_node_slots_connected(
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
        ok_to_add*/
    }
    fn is_edge_reachable(&self, from: &Edge, to: &Edge) -> bool {
        let dag_handle = from.dag_handle.clone();
        let dagnode_handle = NodeHandle::new(dag_handle, None);
        for candidate in self.edges[&from.to_full()].outbound.iter() {
            if self.are_edges_internally_connected(&self.node_data[&candidate.from_full()], &from, &candidate) {
                // See if we can reach `to` from the candidate
                match candidate.to {
                    // The edge we traversed keeps us inside the current DAG
                    Some(_) => if self.is_edge_reachable(candidate, to) {
                        return true
                    },
                    // The edge we traversed takes us out of the DAG.
                    // Consider all nodes aliased to this DAG;
                    //   for each one, consider all paths that lead back to it & continue the
                    //   search.
                    None => {
                        let search = NodeData::Graph(candidate.dag_handle);
                        for node in self.node_data_to_handles(&search) {
                            for edge_out in self.edges[&node].outbound.iter() {
                                // Consider all edges leaving this node that are reachable
                                if edge_out.weight.from_slot == to.weight.to_slot &&
                                    edge_out.weight.from_ch == to.weight.to_ch {
                                    for edge_in in self.paths_from_edge_to_node(edge_out, &node) {
                                        for edge in self.edges[&dagnode_handle].inbound.iter() {
                                            // Follow the edge back into this DAG.
                                            if edge_in.weight.to_slot == edge.weight.from_slot &&
                                                edge_in.weight.to_ch == edge.weight.from_ch {
                                                // Now we're back in the DAG; continue the search
                                                if self.is_edge_reachable(&edge, to) {
                                                    return true
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }
    fn paths_from_edge_to_node<'a>(&'a self, from: &Edge, to: &NodeHandle) -> impl Iterator<Item=Edge> + 'a {
        None.into_iter() // TODO: unimplemented!()
    }
    /// Assuming from.to() == to.from(), will return true if & only if
    /// from and to are internally connected within the node.
    fn are_edges_internally_connected(&self, node_data: &NodeData, from: &Edge, to: &Edge) -> bool {
        unimplemented!()
    }
    fn node_data_to_handles<'a>(&'a self, data: &'a NodeData) -> impl Iterator<Item=NodeHandle> + 'a {
        self.node_data.iter().filter(move |&(handle, node)| {
            node == data
        }).map(|(handle, node)| {
            handle.clone()
        })
    }
    /*fn toplevel_dag(&self) -> &DagImpl {
        &self.dags[&DagHandle{ id: 0 }]
    }*/
    /*fn are_node_slots_connected(&self, data: &NodeData, in_slot: u32, in_ch: u8, out_slot: u32, out_ch: u8) -> bool {
        unimplemented!();
        match *data {
            // TODO: for now, consider all inputs tied to all outputs for each graph.
            // In future, may enforce constraints or actually calculate the connections,
            // but this requires careful planning due to aliasing.
            NodeData::Graph(ref dag_handle) => true,
            NodeData::Effect(ref effect) => effect.are_slots_connected(in_slot, in_ch, out_slot, out_ch)
        }
    }*/
    /// Returns true if there's a path from `in` to `out`.
    pub fn are_slots_connected(&self, in_slot: u32, in_ch: u8, out_slot: u32, out_ch: u8) -> bool {
        unimplemented!();
        /*
        let mut are_connected = false;
        assert!(self.dags.len() == 1); // no nested DAGs
        self.toplevel_dag().traverse(&mut |edge| {
            // ensure we have no nested DAGs.
            assert!(edge.to().map(|to| self.toplevel_dag().node_data(to).is_effect()).unwrap_or(true));
            let do_follow = edge.from() != &None || edge.weight().from_slot == in_slot && edge.weight().from_ch == in_ch;
            if do_follow && edge.weight().to_slot == out_slot && edge.weight().to_ch == out_ch {
                are_connected = true;
            }
            do_follow
        });
        are_connected*/
    }
    pub fn del_node(&mut self, node: NodeHandle) -> Result<(), ()> {
        let ok_to_delete = match self.edges.entry(node) {
            // Already deleted
            hash_map::Entry::Vacant(_) => Ok(()),
            hash_map::Entry::Occupied(entry) => {
                if entry.get().is_empty() {
                    entry.remove();
                    Ok(())
                } else {
                    // Node has edges
                    Err(())
                }
            }
        };
        if let Ok(_) = ok_to_delete {
            // delete the data associated with this node
            self.node_data.remove(&node);
            // notify watchers of successful deletion
            for w in &mut self.watchers {
                w.on_del_node(&node);
            }
        }
        ok_to_delete
    }
    pub fn del_edge(&mut self, edge: Edge) {
        if let Some(edge_set) = self.edges.get_mut(&edge.from_full()) {
            edge_set.outbound.remove(&edge);
        }
        if let Some(edge_set) = self.edges.get_mut(&edge.to_full()) {
            edge_set.inbound.remove(&edge);
        }
        for w in &mut self.watchers {
            w.on_del_edge(&edge);
        }
    }
}

impl NodeHandle {
    fn new(dag: DagHandle, node: Option<PrimNodeHandle>) -> Self {
        Self {
            dag_handle: dag,
            node_handle: node,
        }
    }
    fn dag_handle(&self) -> &DagHandle {
        &self.dag_handle
    }
    fn node_handle(&self) -> &Option<PrimNodeHandle> {
        &self.node_handle
    }
}

impl Edge {
    fn new(dag: DagHandle) -> Self {
        unimplemented!();
        /*Self {
            dag_handle: dag,
            edge: edge,
        }*/
    }
    fn dag_handle(&self) -> &DagHandle {
        &self.dag_handle
    }
    fn from_full(&self) -> NodeHandle {
        NodeHandle {
            dag_handle: self.dag_handle,
            node_handle: self.from,
        }
    }
    fn to_full(&self) -> NodeHandle {
        NodeHandle {
            dag_handle: self.dag_handle,
            node_handle: self.to,
        }
    }
}

impl NodeData {
    fn is_effect(&self) -> bool {
        match *self {
            NodeData::Effect(_) => true,
            _ => false,
        }
    }
}

impl EdgeSet {
    fn new() -> Self {
        EdgeSet {
            outbound: HashSet::new(),
            inbound: HashSet::new(),
        }
    }
    fn is_empty(&self) -> bool {
        self.outbound.is_empty()
    }
}
