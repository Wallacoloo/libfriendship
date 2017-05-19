/// `RouteGraph` defines a Directed Acyclic Graph of Effects.
/// The edges connecting each Effect have a source and destination slot, tag, and channel.
/// Edges are also allowed to go to null, in which case they only have a destination slot and
/// channel. These are outputs.
/// Edges can also COME from null, in which case the source has the format (slot, channel)

use std::collections::hash_map::HashMap;
use std::collections::hash_map;
use std::collections::hash_set::HashSet;
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

/// None represents the Top-level DAG
type DagHandle = NullableInt<u32>;

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
    /// Error inside some Effect:: method
    EffectError(effect::Error),
}

/// Alias for a `Result` with our error type.
pub type ResultE<T> = Result<T, Error>;


#[derive(Default)]
pub struct RouteGraph {
    edges: HashMap<NodeHandle, EdgeSet>,
    node_data: HashMap<NodeHandle, NodeData>,
}

struct EdgeSet {
    outbound: HashSet<Edge>,
    inbound: HashSet<Edge>,
}


impl RouteGraph {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn iter_nodes<'a>(&'a self) -> impl Iterator<Item=(&NodeHandle, &NodeData)> + 'a {
        self.node_data.iter()
    }
    pub fn iter_edges<'a>(&'a self) -> impl Iterator<Item=&Edge> + 'a {
        self.edges.values().flat_map(|v_set| v_set.outbound.iter())
    }
    pub fn get_data(&self, handle: &NodeHandle) -> Option<&NodeData> {
        self.node_data.get(handle)
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
    fn is_edge_reachable(&self, from: &Edge, target: &Edge) -> bool {
        // Algorithm:
        //   Try to reach `edge` from `edge`.
        //   If we reach the boundary of the DAG while doing so, consider all reachable outbound
        //     edges of the DAG
        //     For each such edge, try to reach this DAG (recursively), and then resume the search for `edge`.
        if let Some(_to) = from.to.node_handle.get() {
            // The edge points to a NODE inside a DAG.
            // Consider all (reachable) outgoing edges of the node:
            if let Some(node_data) = self.edges.get(&from.to_full()) {
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
        self.node_data[&from.to_full()]
            .are_slots_connected(from.weight.to_slot, to.weight.from_slot)
    }
    /// Returns true if there's a path from `in` to `out` at the toplevel DAG.
    pub fn are_slots_connected(&self, in_slot: u32, out_slot: u32) -> bool {
        // Consider all edges from None paired with all edges to None:
        let root_dag = NodeHandle::toplevel();
        let edges_from = self.edges[&root_dag].outbound.iter().filter(|&edge| {
            edge.weight.from_slot == in_slot
        });
        for edge_from in edges_from {
            let edges_to = self.edges[&root_dag].inbound.iter().filter(|&edge| {
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
        if ok_to_delete.is_ok() {
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
        // Map Effect -> EffectId
        let nodes = self.node_data.iter().map(|(handle, data)| {
            (*handle, data.id())
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

        // Map EffectId -> Effect
        let nodes: ResultE<HashMap<NodeHandle, NodeData>> = nodes.into_iter().map(|(handle, id)| {
            let decoded_data = Effect::from_id(id, res)?;
            Ok((handle, decoded_data))
        }).collect();
        // Type deduction isn't smart enough to unwrap nodes in above statement.
        let nodes = nodes?;

        // Build self with only nodes and no edges
        let mut me = Self {
            edges: HashMap::new(),
            node_data: nodes,
        };

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


/// Conversion from `effect::Error` for use with the `?` operator
impl From<effect::Error> for Error {
    fn from(e: effect::Error) -> Self {
        Error::EffectError(e)
    }
}
