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
                    self.get_value(edge, time, &vec!())
                }).sum()
            }
        }
    }
}
impl RefRenderer {
    pub fn new() -> Self {
        Self{ nodes: HashMap::new() }
    }
    /// Get the value on an edge at a particular time
    /// When backtracking from the output, we push each Node onto the context if we enter inside of
    ///   it (i.e. if it's a nested DAG) & pop when exiting.
    fn get_value(&self, edge: &Edge, time: u64, context: &Vec<NodeHandle>) -> f32 {
        let from = edge.from_full();
        if *from.node_handle() == None {
            // Reading from one of the inputs to the top of `context`
            let mut new_context = context.clone();
            let head = new_context.pop().unwrap();
            // Sum the inputs to the matchin slot/ch
            let in_edges = self.nodes[&head].inbound.iter().filter(|in_edge| {
                in_edge.to_slot() == edge.from_slot() && in_edge.to_ch() == edge.from_ch()
            });
            let in_values = in_edges.map(|in_edge| {
                self.get_value(in_edge, time, &new_context)
            });
            in_values.sum()
        } else {
            // Reading from another node within the DAG
            let node = &self.nodes[&from];
            match node.data {
                MyNodeData::UserNode(ref effect) => unimplemented!(),
                // Output = sum of all edges to Null of the same slot & ch, within the given DAG.
                MyNodeData::Graph(ref dag_handle) => {
                    let mut new_context = context.clone();
                    new_context.push(from);
                    let subdag = &self.nodes[&NodeHandle::new(dag_handle.clone(), None)];
                    let out_edges = subdag.inbound.iter().filter(|out_edge| {
                        out_edge.to_slot() == edge.from_slot() && out_edge.to_ch() == edge.from_ch()
                    });
                    let out_values = out_edges.map(|out_edge| {
                        self.get_value(out_edge, time, &new_context)
                    });
                    out_values.sum()
                }
                // Output = sum of all inputs to slot 0 of the same ch.
                MyNodeData::Delay(ref frames) => {
                    assert!(edge.from_slot() == 1);
                    let in_edges = node.inbound.iter().filter(|in_edge| {
                        in_edge.to_slot() == 0 && in_edge.to_ch() == edge.from_ch()
                    });
                    let in_values = in_edges.map(|in_edge| {
                        self.get_value(in_edge, time, context)
                    });
                    in_values.sum()
                },
                MyNodeData::Constant(ref value) => value.clone(),
            }
        }
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
