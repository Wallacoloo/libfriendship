/// `RouteGraph` defines a Directed Acyclic Graph of Effects.
/// The edges connecting each Effect have a source and destination slot.
/// Edges are also allowed to go to null, in which case they are treated as outputs.
/// Edges can also come from null, in which case they are treated as inputs.

use std::collections::hash_map::HashMap;
use std::collections::hash_map;
use std::collections::hash_set::HashSet;
use std::fmt::{Display, Formatter};
use std::fmt;
use std::ops::Deref;
use std::rc::Rc;

use resman::ResMan;
use super::adjlist::AdjList;
use super::effect;
use super::effect::Effect;
use super::nullable_int::NullableInt;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct EdgeWeight {
    from_slot: u32,
    to_slot: u32,
}

pub type NodeData = Rc<Effect>;

/// None represents the Dag's I/O
type PrimNodeHandle = NullableInt<u32>;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct NodeHandle {
    node_handle: PrimNodeHandle,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct Edge {
    from: NodeHandle,
    to: NodeHandle,
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
    /// Raised on attempt to wire two edges to the same input slot (i.e. many-to-one).
    SlotAlreadyConnected,
    /// Raised on attempt to connect an edge to/from a nonexistant node.
    NoSuchNode,
    /// Raised on attempt to connect an edge to/from a nonexistent slot.
    NoSuchSlot,
    /// Error inside some Effect:: method
    EffectError(effect::Error),
}

/// Alias for a `Result` with our error type.
pub type ResultE<T> = Result<T, Error>;


#[derive(Debug)]
pub struct RouteGraph {
    /// Associate node handles with their data.
    nodes: HashMap<NodeHandle, Node>,
}

#[derive(Debug)]
struct Node {
    outbound: HashSet<Edge>,
    inbound: HashSet<Edge>,
    node_data: Option<NodeData>,
}


impl Default for RouteGraph {
    fn default() -> Self {
        let mut nodes = HashMap::new();
        // allocate space for toplevel I/Os
        nodes.insert(NodeHandle::toplevel(), Node::null());
        Self { nodes }
    }
}
impl RouteGraph {
    pub fn new() -> Self {
        Default::default()
    }
    /// Iterate over the nodes in an unordered way.
    pub fn iter_nodes<'a>(&'a self) -> impl Iterator<Item=(&NodeHandle, &NodeData)> + 'a {
        self.nodes.iter().filter_map(|(hnd, node)| {
            node.node_data.as_ref().map(|data| {
                (hnd, data)
            })
        })
    }
    /// Iterate over the nodes in such an order that by the time each node it
    /// visited, all of the nodes that have edges going *into* it have already
    /// been visited.
    pub fn iter_nodes_dep_first<'a>(&'a self) -> impl Iterator<Item=NodeHandle> + 'a {
        let mut visited = HashSet::new();
        let mut ordered = Vec::new();
        for (node, _data) in self.iter_nodes() {
            self.dep_first_helper(&mut visited, &mut ordered, *node);
        }
        ordered.into_iter()
    }
    fn dep_first_helper(&self, visited: &mut HashSet<NodeHandle>, ordered: &mut Vec<NodeHandle>, node_hnd: NodeHandle) {
        if !node_hnd.is_toplevel() {
            if let Some(node) = self.nodes.get(&node_hnd) {
                // ensure all dependencies have been visited
                for dep_edge in node.inbound.iter() {
                    self.dep_first_helper(visited, ordered, dep_edge.from_full());
                }
            }
            if visited.insert(node_hnd) {
                // Node hasn't been seen
                ordered.push(node_hnd);
            }
        }
    }
    /// Iterate over all edges in an unordered way.
    pub fn iter_edges<'a>(&'a self) -> impl Iterator<Item=&Edge> + 'a {
        self.nodes.values().flat_map(|v_set| v_set.outbound.iter())
    }
    /// Iterate over the edges that point into outputs, in an unordered way.
    pub fn iter_outbound_edges<'a>(&'a self) -> impl Iterator<Item=&Edge> + 'a {
        // TODO: the graph is bidirectional - can we just look at the inputs to NULL?
        self.iter_edges().filter(|e| {
            e.to_full().is_toplevel()
        })
    }
    /// Retrieve the data associated with a node, or `None` if the node handle
    /// does not exist within this graph.
    pub fn get_data(&self, handle: &NodeHandle) -> Option<&NodeData> {
        self.nodes.get(handle).and_then(|node| node.node_data.as_ref())
    }
    /// Iterate over all the edges leading directly into the given node.
    pub fn iter_edges_to<'a>(&'a self, handle: &NodeHandle) -> impl Iterator<Item=&Edge> + 'a {
        self.nodes.get(handle).map(|node| {
            node.inbound.iter()
        }).into_iter().flat_map(|i| i)
    }
    /// Try to create a node with the given handle/data.
    /// Will error if the handle is already in use.
    pub fn add_node(&mut self, handle: NodeHandle, node_data: NodeData) -> ResultE<()> {
        // Create storage for the node's outgoing edges
        match self.nodes.entry(handle) {
            hash_map::Entry::Occupied(_) => Err(Error::NodeExists),
            hash_map::Entry::Vacant(entry) => {
                entry.insert(Node::new(Some(node_data)));
                Ok(())
            },
        }
    }
    /// Connect two nodes with an edge.
    /// Will return an error if the connection would violate any of the DAGs constraints.
    pub fn add_edge(&mut self, edge: Edge) -> ResultE<()> {
        // Each node input may only have one inbound edge.
        if let hash_map::Entry::Occupied(entry) = self.nodes.entry(edge.to_full()) {
            let is_slot_in_use = entry.get().inbound.iter()
                .filter(|in_edge| in_edge.to_slot() == edge.to_slot())
                .next().is_some();
            if is_slot_in_use {
                return Err(Error::SlotAlreadyConnected);
            }
        }
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
    /// Functionally equivalent to the `add_edge` method, but does not validate DAG constraints.
    fn add_edge_unchecked(&mut self, edge: Edge) {
        // associate the edge with its origin.
        self.nodes.get_mut(&edge.from_full()).unwrap().outbound.insert(edge.clone());
        // associate the edge with its destination.
        self.nodes.get_mut(&edge.to_full()).unwrap().inbound.insert(edge);
    }
    /// Returns true if there is some directed path the connects `from` to `target`.
    /// Note that neither edge need currently exist in the graph.
    fn is_edge_reachable(&self, from: &Edge, target: &Edge) -> bool {
        // Algorithm:
        //   Try to reach `edge` from `edge`.
        //   If we reach the boundary of the DAG while doing so, consider all reachable outbound
        //     edges of the DAG
        //     For each such edge, try to reach this DAG (recursively), and then resume the search for `edge`.
        if let Some(_to) = from.to.node_handle.get() {
            // The edge points to a NODE inside a DAG.
            // Consider all (reachable) outgoing edges of the node:
            if let Some(node_data) = self.nodes.get(&from.to_full()) {
                for candidate_edge in &node_data.outbound {
                    if self.are_edges_internally_connected(from, candidate_edge) && 
                      self.is_edge_reachable(candidate_edge, target) {
                        return true;
                    }
                }
            }
        }
        false
    }
    /// Assuming from.to() == to.from(), will return true if & only if
    /// from and to are internally connected within the node.
    fn are_edges_internally_connected(&self, from: &Edge, to: &Edge) -> bool {
        self.nodes[&from.to_full()].node_data.as_ref().unwrap()
            .are_slots_connected(from.weight.to_slot, to.weight.from_slot)
    }
    /// Returns true if there's a path from `in` to `out` at the toplevel DAG.
    pub fn are_slots_connected(&self, in_slot: u32, out_slot: u32) -> bool {
        // Consider all edges from None paired with all edges to None:
        let root_dag = NodeHandle::toplevel();
        let edges_from = self.nodes[&root_dag].outbound.iter().filter(|&edge| {
            edge.weight.from_slot == in_slot
        });
        for edge_from in edges_from {
            let edges_to = self.nodes[&root_dag].inbound.iter().filter(|&edge| {
                edge.weight.to_slot == out_slot
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
        match self.nodes.entry(node) {
            // Already deleted
            hash_map::Entry::Vacant(_) => Ok(()),
            hash_map::Entry::Occupied(entry) => {
                if entry.get().has_no_edges() {
                    entry.remove();
                    Ok(())
                } else {
                    // Node has edges
                    Err(Error::NodeInUse)
                }
            }
        }
    }
    pub fn del_edge(&mut self, edge: Edge) {
        if let Some(edge_set) = self.nodes.get_mut(&edge.from_full()) {
            edge_set.outbound.remove(&edge);
        }
        if let Some(edge_set) = self.nodes.get_mut(&edge.to_full()) {
            edge_set.inbound.remove(&edge);
        }
    }
    // TODO: replace this with an implementation of `Into`
    pub fn to_adjlist(&self) -> AdjList {
        // Map Effect -> EffectId
        let nodes = self.nodes.iter().filter_map(|(handle, ref data)| {
            data.node_data.as_ref().map(|node_data| {
                (*handle, node_data.id().clone())
            })
        }).collect();
        // Doubly-linked edges -> singly-linked
        let edges = self.nodes.iter().flat_map(|(_key, edgeset)| {
            edgeset.outbound.clone().into_iter()
        }).collect();

        AdjList {
            nodes: nodes,
            edges: edges,
        }
    }
    // TODO: replace with an implementation of `TryFrom`.
    pub fn from_adjlist(adj: AdjList, res: &ResMan) -> ResultE<Self> {
        // Unwrap struct fields to local variables
        let (nodes, edges) = (adj.nodes, adj.edges);

        // Map EffectId -> Effect
        let nodes: ResultE<HashMap<NodeHandle, Node>> = nodes.into_iter().map(|(handle, id)| {
            let decoded_data = Effect::from_id(id, res)?;
            Ok((handle, Node::new(Some(decoded_data))))
        }).collect();
        // Type deduction isn't smart enough to unwrap nodes in above statement.
        let mut nodes = nodes?;
        nodes.insert(NodeHandle::toplevel(), Node::null());

        // Build self with only nodes and no edges
        let mut me = Self { nodes };

        // Add the edges one at a time, enforcing zero cycles
        for edge in &edges {
            me.add_edge(edge.clone())?
        }
        Ok(me)
    }
}

impl NodeHandle {
    pub fn toplevel() -> Self {
        NodeHandle::new(None)
    }
    pub fn new<T>(node_handle: T) -> Self
        where T: Into<PrimNodeHandle>
    {
        Self{ node_handle: node_handle.into() }
    }
    pub fn node_handle(&self) -> &PrimNodeHandle {
        &self.node_handle
    }
    pub fn is_toplevel(&self) -> bool {
        *self.node_handle() == None
    }
}
impl Display for NodeHandle {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.node_handle.fmt(f)
    }
}
impl Deref for NodeHandle {
    type Target = PrimNodeHandle;
    fn deref(&self) -> &Self::Target {
        &self.node_handle
    }
}

impl Edge {
    /// Create an edge from `from` to null (i.e. an output)
    pub fn new_to_null(from: NodeHandle, weight: EdgeWeight) -> Self {
        Self {
            from,
            to: NodeHandle::toplevel(),
            weight,
        }
    }
    pub fn new_from_null(to: NodeHandle, weight: EdgeWeight) -> Self {
        Self {
            from: NodeHandle::toplevel(),
            to,
            weight
        }
    }
    /// Create an edge between the two nodes.
    pub fn new(from: NodeHandle, to: NodeHandle, weight: EdgeWeight) -> Self {
        Self{ from, to, weight }
    }
    // TODO: rename to 'from' and 'to' or 'from_hnd', 'to_hnd'?
    pub fn from_full(&self) -> NodeHandle {
        self.from
    }
    pub fn to_full(&self) -> NodeHandle {
        self.to
    }
    pub fn to_slot(&self) -> u32 {
        self.weight.to_slot
    }
    pub fn from_slot(&self) -> u32 {
        self.weight.from_slot
    }
}

impl EdgeWeight {
    pub fn new(from_slot: u32, to_slot: u32) -> Self {
        Self{ from_slot, to_slot }
    }
}


impl Node {
    fn new(node_data: Option<NodeData>) -> Self {
        Self {
            node_data,
            outbound: HashSet::new(),
            inbound: HashSet::new(),
        }
    }
    fn null() -> Self {
        Self::new(None)
    }
    fn has_no_edges(&self) -> bool {
        self.outbound.is_empty() && self.inbound.is_empty()
    }
}


/// Conversion from `effect::Error` for use with the `?` operator
impl From<effect::Error> for Error {
    fn from(e: effect::Error) -> Self {
        Error::EffectError(e)
    }
}
