use std::collections::{HashMap, HashSet};

use render::Renderer;
use routing::{Edge, GraphWatcher, NodeData, NodeHandle};

pub struct RefRenderer {
    nodes: HashMap<NodeHandle, Node>,
}

struct Node {
    data: NodeData,
    inbound: HashSet<Edge>
}

impl Renderer for RefRenderer {
    fn get_sample(&mut self, time: u64, ch: u8) -> f32 {
        // Try to find the edge that goes to -> (Null, slot=0, ch=ch)
        let root_handle = NodeHandle::toplevel();
        match self.nodes.get(&root_handle) {
            // for an empty graph, yield silence.
            None => 0f32,
            Some(node) => {
                // find all edges to ([Null], slot=0, ch=ch)
                let matches = node.inbound.iter().filter(|edge| {
                    edge.to_slot() == 0 && edge.to_ch() == ch
                });
                matches.map(|edge| {
                    self.get_value(edge, time)
                }).sum()
            }
        }
    }
}
impl RefRenderer {
    /// Get the value on an edge at a particular time
    fn get_value(&self, edge: &Edge, time: u64) -> f32 {
        unimplemented!()
    }
}

impl GraphWatcher for RefRenderer {
    fn on_add_node(&mut self, handle: &NodeHandle, data: &NodeData) {
        self.nodes.insert(handle.clone(), Node::new(data.clone()));
    }
    fn on_del_node(&mut self, handle: &NodeHandle) {
        self.nodes.remove(handle);
    }
    fn on_add_edge(&mut self, edge: &Edge) {
        self.nodes.get_mut(&edge.to_full()).unwrap().inbound.insert(edge.clone());
    }
    fn on_del_edge(&mut self, edge: &Edge) {
        self.nodes.get_mut(&edge.to_full()).unwrap().inbound.remove(edge);
    }
}


impl Node {
    fn new(data: NodeData) -> Self {
        Node {
            data: data,
            inbound: HashSet::new(),
        }
    }
}
