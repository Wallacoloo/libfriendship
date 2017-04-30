/// RouteGraph defines a Directed Acyclic Graph of Effects.
/// The edges connecting each Effect have a source and destination slot, tag, and channel.
/// Edges are also allowed to go to null, in which case they only have a destination slot and
/// channel. These are outputs.
/// Edges can also COME from null, in which case the source has the format (slot, channel)

use std::cmp;
use std::collections::hash_map::HashMap;
use std::collections::hash_map;
use std::collections::hash_set::HashSet;
use std::rc::Rc;

use resman::ResMan;
use super::adjlist::AdjList;
use super::adjlist;
use super::effect;
use super::effect::Effect;

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
#[derive(Serialize, Deserialize)]
struct EdgeWeight {
    from_slot: u32,
    from_ch: u8,
    to_slot: u32,
    to_ch: u8,
}

#[derive(Clone, Eq, PartialEq)]
pub enum NodeData {
    Effect(Rc<Effect>),
    Graph(DagHandle),
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct DagHandle {
    // None represents the Top-level DAG
    id: Option<u32>,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct PrimNodeHandle {
    id: u64,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct NodeHandle {
    dag_handle: DagHandle,
    node_handle: Option<PrimNodeHandle>,
}

#[derive(Clone, Hash, Eq, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct Edge {
    dag_handle: DagHandle,
    from: Option<PrimNodeHandle>,
    to: Option<PrimNodeHandle>,
    weight: EdgeWeight,
}

#[derive(Debug)]
pub enum Error {
    /// Raised when an attempt to modify the graph would create a dependency cycle.
    WouldCycle,
    /// Raised on attempt to delete a node when it still has edges.
    NodeInUse,
    /// Raised on attempt to create a node with an id that's already in use.
    NodeExists,
    /// Error inside some Effect:: method
    EffectError(effect::Error),
}

/// Alias for a `Result` with our error type.
pub type ResultE<T> = Result<T, Error>;


pub struct RouteGraph {
    // TODO: can make these non-zero for more efficient Option<NodeHandle> encoding
    dag_counter: u32,
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
            edges: HashMap::new(),
            node_data: HashMap::new(),
        }
    }
    pub fn iter_nodes<'a>(&'a self) -> impl Iterator<Item=&NodeHandle> + 'a {
        self.node_data.keys()
    }
    pub fn iter_edges<'a>(&'a self) -> impl Iterator<Item=&Edge> + 'a {
        self.edges.values().flat_map(|v_set| v_set.outbound.iter())
    }
    pub fn add_dag(&mut self) -> DagHandle {
        let handle = DagHandle{ id: Some(self.dag_counter) };
        self.dag_counter += 1;
        handle
    }
    /// Try to create a node with the given handle/data.
    /// Will error if the handle is already in use.
    pub fn add_node(&mut self, handle: NodeHandle, node_data: NodeData) -> ResultE<()> {
        // Create storage for the node's outgoing edges
        match self.edges.entry(handle) {
            hash_map::Entry::Occupied(_) => Err(Error::NodeExists),
            hash_map::Entry::Vacant(entry) => { entry.insert(EdgeSet::new()); Ok(()) },
        }?;
        // Store the node's data
        assert!(self.node_data.insert(handle, node_data.clone()).is_none());
        Ok(())
    }
    pub fn add_edge(&mut self, edge: Edge) -> ResultE<()> {
        // Algorithm:
        //   Assume we currently have a DAG.
        //   Given that, the only way this new edge could introduce a cycle is if it was a part of
        //     that cycle.
        //   Therefore, if no path exists from the edge to itself, then it is safe to add the edge.
        let is_reachable = self.is_edge_reachable(&edge, &edge);
        if is_reachable {
            Err(Error::WouldCycle)
        } else {
            self.add_edge_unchecked(edge);
            Ok(())
        }
    }
    fn add_edge_unchecked(&mut self, edge: Edge) {
        // associate the edge with its origin.
        self.edges.entry(edge.from_full()).or_insert_with(EdgeSet::new).outbound.insert(edge.clone());
        // associate the edge with its destination.
        self.edges.entry(edge.to_full()).or_insert_with(EdgeSet::new).inbound.insert(edge);
    }
    fn is_edge_reachable(&self, from: &Edge, to: &Edge) -> bool {
        // Algorithm:
        //   Try to reach `edge` from `edge`.
        //   If we reach the boundary of the DAG while doing so, consider all reachable outbound
        //     edges of the DAG
        //     For each such edge, try to reach this DAG (recursively), and then resume the search for `edge`.
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
    /// Return all edges into `to` that are reachable from `from`.
    fn paths_from_edge_to_node<'a>(&'a self, from: &'a Edge, to: &'a NodeHandle) -> impl Iterator<Item=&'a Edge> + 'a {
        self.edges[to].inbound.iter().filter(move |e| {
            self.is_edge_reachable(&from, e)
        })
    }
    /// Assuming from.to() == to.from(), will return true if & only if
    /// from and to are internally connected within the node.
    fn are_edges_internally_connected(&self, node_data: &NodeData, from: &Edge, to: &Edge) -> bool {
        match *node_data {
            NodeData::Effect(ref effect) => effect.are_slots_connected(
                from.weight.to_slot, from.weight.to_ch,
                to.weight.from_slot, to.weight.from_ch),
            // See if there's a path from (None->from.to) to (to.from->None) within the dag
            NodeData::Graph(ref dag_handle) => {
                let dagnode_handle = NodeHandle::new(dag_handle.clone(), None);
                // Consider all edges from (None->from.to)
                self.edges[&dagnode_handle].outbound.iter().filter(|new_from| {
                    new_from.weight.from_slot == from.weight.to_slot &&
                        new_from.weight.from_ch == from.weight.to_ch
                })
                // Check if there's a path to (None) and that the edge to (None) is (to.from->None)
                .any(|new_from| {
                    self.paths_from_edge_to_node(new_from, &dagnode_handle).any(|new_to| {
                        new_to.weight.to_slot == to.weight.from_slot &&
                            new_to.weight.to_ch == to.weight.from_ch
                    })
                })
            }
        }
    }
    fn node_data_to_handles<'a>(&'a self, data: &'a NodeData) -> impl Iterator<Item=NodeHandle> + 'a {
        self.node_data.iter().filter(move |&(_handle, node)| {
            node == data
        }).map(|(handle, _node)| {
            handle.clone()
        })
    }
    /// Returns true if there's a path from `in` to `out` at the toplevel DAG.
    pub fn are_slots_connected(&self, in_slot: u32, in_ch: u8, out_slot: u32, out_ch: u8) -> bool {
        // Consider all edges from None paired with all edges to None:
        let root_dag = NodeHandle::new(DagHandle { id: None }, None);
        let edges_from = self.edges[&root_dag].outbound.iter().filter(|&edge| {
            edge.weight.from_slot == in_slot && edge.weight.from_ch == in_ch
        });
        for edge_from in edges_from {
            let edges_to = self.edges[&root_dag].inbound.iter().filter(|&edge| {
                edge.weight.to_slot == out_slot && edge.weight.to_ch == out_ch
            });
            for edge_to in edges_to {
                if self.is_edge_reachable(edge_from, edge_to) {
                    return true;
                }
            }
        }
        false
    }
    pub fn del_node(&mut self, node: NodeHandle) -> ResultE<()> {
        let ok_to_delete = match self.edges.entry(node) {
            // Already deleted
            hash_map::Entry::Vacant(_) => Ok(()),
            hash_map::Entry::Occupied(entry) => {
                if entry.get().is_empty() {
                    entry.remove();
                    Ok(())
                } else {
                    // Node has edges
                    Err(Error::NodeInUse)
                }
            }
        };
        if let Ok(_) = ok_to_delete {
            // delete the data associated with this node
            self.node_data.remove(&node);
        }
        ok_to_delete
    }
    pub fn del_edge(&mut self, edge: Edge) {
        // TODO: garbage collect the edge sets.
        if let Some(edge_set) = self.edges.get_mut(&edge.from_full()) {
            edge_set.outbound.remove(&edge);
        }
        if let Some(edge_set) = self.edges.get_mut(&edge.to_full()) {
            edge_set.inbound.remove(&edge);
        }
    }

    pub fn to_adjlist(&self) -> AdjList {
        // Map Effect -> EffectMeta
        let nodes = self.node_data.iter().map(|(handle, data)| {
            (handle.clone(), data.to_adjlist_data())
        }).collect();
        // Doubly-linked edges -> singly-linked
        let edges = self.edges.iter().flat_map(|(_key, edgeset)| {
            edgeset.outbound.clone().into_iter()
        }).collect();

        AdjList {
            nodes: nodes,
            edges: edges,
        }
    }
    pub fn from_adjlist(adj: AdjList, res: &ResMan) -> ResultE<Self> {
        // Unwrap struct fields to local variables
        let (nodes, edges) = (adj.nodes, adj.edges);

        // Map EffectMeta -> Effect and also determine the highest ids in use
        let mut dag_counter = 0;
        let nodes: ResultE<HashMap<NodeHandle, NodeData>> = nodes.into_iter().map(|(handle, data)| {
            if let Some(dag_hnd) = handle.dag_handle.id {
                dag_counter = cmp::max(dag_counter, dag_hnd);
            }
            let decoded_data = match data {
                adjlist::NodeData::Effect(meta) =>
                    NodeData::Effect(Effect::from_meta(meta, res)?),
                adjlist::NodeData::Graph(dag_handle) =>
                    NodeData::Graph(dag_handle),
            };
            Ok((handle.clone(), decoded_data))
        }).collect();
        // Type deduction isn't smart enough to unwrap nodes in above statement.
        let nodes = nodes?;

        // Build self with only nodes and no edges
        let mut me = Self {
            dag_counter: dag_counter,
            edges: HashMap::new(),
            node_data: nodes,
        };

        // Add the edges one at a time, enforcing zero cycles
        for edge in edges.into_iter() {
            me.add_edge(edge)?
        }
        Ok(me)
    }
}

impl NodeHandle {
    pub fn toplevel() -> Self {
        NodeHandle::new(DagHandle::toplevel(), None)
    }
    pub fn new(dag: DagHandle, node: Option<PrimNodeHandle>) -> Self {
        Self {
            dag_handle: dag,
            node_handle: node,
        }
    }
    pub fn dag_handle(&self) -> &DagHandle {
        &self.dag_handle
    }
    pub fn node_handle(&self) -> &Option<PrimNodeHandle> {
        &self.node_handle
    }
}

impl Edge {
    fn dag_handle(&self) -> &DagHandle {
        &self.dag_handle
    }
    pub fn from_full(&self) -> NodeHandle {
        NodeHandle {
            dag_handle: self.dag_handle,
            node_handle: self.from,
        }
    }
    pub fn to_full(&self) -> NodeHandle {
        NodeHandle {
            dag_handle: self.dag_handle,
            node_handle: self.to,
        }
    }
    pub fn to_slot(&self) -> u32 {
        self.weight.to_slot
    }
    pub fn to_ch(&self) -> u8 {
        self.weight.to_ch
    }
    pub fn from_slot(&self) -> u32 {
        self.weight.from_slot
    }
    pub fn from_ch(&self) -> u8 {
        self.weight.from_ch
    }
}

impl NodeData {
    fn is_effect(&self) -> bool {
        match *self {
            NodeData::Effect(_) => true,
            _ => false,
        }
    }
    /// NodeData normally encodes references to actual node implementations -
    /// in order to know their internal connections, etc.
    /// This transforms it into a type that is suitable for transmission, i.e.
    /// metadata explaining how to locate the correct effect implementation.
    fn to_adjlist_data(&self) -> adjlist::NodeData {
        match *self {
            NodeData::Effect(ref effect) => adjlist::NodeData::Effect(effect.meta().clone()),
            NodeData::Graph(ref dag) => adjlist::NodeData::Graph(dag.clone()),
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
        self.outbound.is_empty() && self.inbound.is_empty()
    }
}

impl DagHandle {
    fn toplevel() -> Self {
        DagHandle {
            id: None
        }
    }
}



/// Conversion from `effect::Error` for use with the `?` operator
impl From<effect::Error> for Error {
    fn from(e: effect::Error) -> Self {
        Error::EffectError(e)
    }
}
