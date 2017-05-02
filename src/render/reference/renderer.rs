use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map;
use std::rc::Rc;

use render::Renderer;
use routing::{DagHandle, Edge, Effect, GraphWatcher, NodeData, NodeHandle};

#[derive(Default)]
pub struct RefRenderer {
    nodes: HashMap<NodeHandle, Node>,
}

struct Node {
    data: MyNodeData,
    inbound: HashSet<Edge>
}

enum MyNodeData {
    /// This node is a non-primitive effect.
    UserNode(Rc<Effect>),
    /// This node is an instance of another DAG.
    Graph(DagHandle),
    /// Primitive Delay(samples) effect
    Delay(u64),
    /// Primitive Constant(value) effect.
    /// Also serves as a unit step;
    /// Returns the f32 value for t >= 0, else 0.
    Constant(f32),
    /// Primitive effect to multiply TWO input streams sample-wise.
    Multiply,
    /// This node is a DAG definition. i.e. it holds the output edges of a DAG.
    DagIO,
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
                self.sum_input_to_slot(node, time, 0, ch, &Vec::new())
            }
        }
    }
}
impl RefRenderer {
    /// Get the value on an edge at a particular time
    /// When backtracking from the output, we push each Node onto the context if we enter inside of
    ///   it (i.e. if it's a nested DAG) & pop when exiting.
    fn get_value(&self, edge: &Edge, time: u64, context: &Vec<NodeHandle>) -> f32 {
        let from = edge.from_full();
        if *from.node_handle() == None {
            // Reading from one of the inputs to the top of `context`
            let mut new_context = context.clone();
            let head = new_context.pop().unwrap();
            // Sum the inputs to the matching slot/ch
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
                MyNodeData::UserNode(ref _effect) => unimplemented!(),
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
                    // The only nonzero output is slot=1.
                    if edge.from_slot() != 1 {
                        println!("Warning: attempt to read from Delay slot != 1");
                        0f32
                    } else {
                        match time.checked_sub(*frames) {
                            // t < 0 => no audio
                            None => 0f32,
                            Some(origin_time) => self.sum_input_to_slot(node, origin_time, 0, edge.from_ch(), context)
                        }
                    }
                },
                MyNodeData::Constant(ref value) => {
                    // The only nonzero output is slot=1.
                    if edge.from_slot() != 1 {
                        println!("Warning: attempt to read from Constant slot != 1");
                        0f32
                    } else {
                        *value
                    }
                },
                MyNodeData::Multiply => {
                    // The only nonzero output is slot=1.
                    if edge.from_slot() != 1 {
                        println!("Warning: attempt to read from Multiply slot != 1");
                        0f32
                    } else {
                        // Sum all inputs from slot=0 and slot=2 into two separate
                        // variables, then multiply them.
                        let val_a = self.sum_input_to_slot(node, time, 0, edge.from_ch(), context);
                        let val_b = self.sum_input_to_slot(node, time, 2, edge.from_ch(), context);
                        val_a * val_b
                    }
                },
                _ => panic!("Internal RefRenderer error: illegal node type"),
            }
        }
    }
    /// Return the sum of all inputs into a specific slot/channel of the given
    /// node at the given time.
    fn sum_input_to_slot(&self, node: &Node, time: u64, slot: u32, ch: u8, context: &Vec<NodeHandle>) -> f32 {
        let edges_in = node.inbound.iter().filter(|in_edge| {
            in_edge.to_slot() == slot && in_edge.to_ch() == ch
        });
        edges_in.map(|edge| self.get_value(edge, time, context)).sum()
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
                            "/Multiply" => MyNodeData::Multiply,
                            _ => panic!("Unrecognized primitive effect: {} (full url: {})", url.path(), url),
                        }
                    }
                    None => MyNodeData::UserNode(effect.clone())
                }
            }
        };
        self.nodes.insert(handle.clone(), Node::new(my_node_data));
        // If the node is part of a new DAG, allocate data so that future edges
        // to null within the DAG can be held.
        self.nodes.entry(NodeHandle::new_dag(handle.dag_handle().clone())).or_insert_with(|| {
            Node::new(MyNodeData::DagIO)
        });
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
