use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map;
use std::rc::Rc;

use render::Renderer;
use routing::{DagHandle, Edge, Effect, GraphWatcher, NodeData, NodeHandle};

pub struct RefRenderer {
    nodes: HashMap<NodeHandle, Node>,
}

struct Node {
    data: MyNodeData,
    inbound: HashSet<Edge>
}

enum MyNodeData {
    UserNode(Rc<Effect>),
    Graph(DagHandle),
    /// Primitive Delay(samples) effect
    Delay(u64),
    /// Primitive Constant(value) effect,
    Constant(f32),
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
        let my_node_data = match *data {
            NodeData::Graph(ref handle) => MyNodeData::Graph(handle.clone()),
            NodeData::Effect(ref effect) => {
                match effect.meta().get_primitive_url() {
                    Some(ref url) => {
                        let mut params: HashMap<_, _> = url.query_pairs().collect();
                        match url.path() {
                            "/Delay" => {
                                let frames: u64 = match params.entry(Cow::from("frames")) {
                                    hash_map::Entry::Occupied(e) => e.remove().parse().unwrap(),
                                    hash_map::Entry::Vacant(_) => 0u64,
                                };
                                // Make sure we consumed all arguments.
                                assert!(params.is_empty());
                                MyNodeData::Delay(frames)
                            },
                            "/Constant" => {
                                let value: f32 = match params.entry(Cow::from("value")) {
                                    hash_map::Entry::Occupied(e) => e.remove().parse().unwrap(),
                                    hash_map::Entry::Vacant(_) => 0f32,
                                };
                                // Make sure we consumed all arguments.
                                assert!(params.is_empty());
                                MyNodeData::Constant(value)
                            },
                            _ => panic!("Unrecognized primitive effect"),
                        }
                    }
                    None => MyNodeData::UserNode(effect.clone())
                }
            }
        };
        self.nodes.insert(handle.clone(), Node::new(my_node_data));
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
    fn new(data: MyNodeData) -> Self {
        Node {
            data: data,
            inbound: HashSet::new(),
        }
    }
}
