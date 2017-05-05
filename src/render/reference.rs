use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map;

use render::Renderer;
use routing::{DagHandle, Edge, GraphWatcher, NodeData, NodeHandle};

type NodeMap = HashMap<NodeHandle, Node>;

#[derive(Default)]
pub struct RefRenderer {
    nodes: NodeMap,
}

struct Node {
    data: MyNodeData,
    inbound: HashSet<Edge>
}

enum MyNodeData {
    /// This node is a non-primitive effect.
    UserNode(NodeMap),
    /// This node is an instance of another DAG.
    Graph(DagHandle),
    /// Primitive Delay(samples) effect
    Delay(u64),
    /// Primitive Constant(value) effect.
    /// Also serves as a unit step;
    /// Returns the float value for t >= 0, else 0.
    Constant(f32),
    /// Primitive effect to multiply TWO input streams sample-wise.
    Multiply,
    /// Primitive effect to calculate 1/A
    MultInv,
    /// Primitive effect to calculate A%1 (i.e. A - A.floor())
    ModuloOne,
    /// This node is a DAG definition. i.e. it holds the output edges of a DAG.
    DagIO,
}

impl Renderer for RefRenderer {
    fn get_sample(&mut self, time: u64, ch: u8) -> f32 {
        // Try to find the edge that goes to -> (Null, slot=0, ch=ch)
        let root_handle = NodeHandle::toplevel();
        // empty graph is 0 = silence
        self.nodes.get(&root_handle).map_or(0f32, |node| {
            // find all edges to ([Null], slot=0, ch=ch)
            self.sum_input_to_slot(&self.nodes, node, time, 0, ch, &Vec::new()) as f32
        })
    }
}
impl RefRenderer {
    /// Get the value on an edge at a particular time
    /// When backtracking from the output, we push each Node onto the context if we enter inside of
    ///   it (i.e. if it's a nested DAG) & pop when exiting.
    fn get_value(&self, nodes: &NodeMap, edge: &Edge, time: u64, context: &Vec<(&NodeMap, NodeHandle)>) -> f64 {
        let from = edge.from_full();
        if *from.node_handle() == None {
            // Reading from one of the inputs to the top of `context`
            // TODO: we can avoid cloning by reversing the pop after recursing.
            let mut new_context = context.clone();
            let (new_nodes, head) = new_context.pop().unwrap();
            // Sum the inputs to the matching slot/ch
            self.sum_input_to_slot(new_nodes, &new_nodes[&head], time, edge.from_slot(), edge.from_ch(), &new_context)
        } else {
            // Reading from another node within the DAG
            let node = &nodes[&from];
            match node.data {
                MyNodeData::UserNode(ref new_nodes) => {
                    let mut new_context = context.clone();
                    new_context.push((nodes, from));
                    // Now find the *output* of the sub dag (or 0 if the sub dag has no outputs)
                    new_nodes.get(&NodeHandle::toplevel()).map_or(0f64, |root_node| {
                        self.sum_input_to_slot(&new_nodes, root_node, time, edge.from_slot(), edge.from_ch(), &new_context)
                    })
                },
                // Output = sum of all edges to Null of the same slot & ch, within the given DAG.
                MyNodeData::Graph(ref dag_handle) => {
                    // TODO: we can avoid cloning by reversing the push after recursing.
                    let mut new_context = context.clone();
                    new_context.push((nodes, from));
                    let subdag = &nodes[&NodeHandle::new_dag(dag_handle.clone())];
                    self.sum_input_to_slot(nodes, &subdag, time, edge.from_slot(), edge.from_ch(), &new_context)
                }
                // Output = sum of all inputs to slot 0 of the same ch.
                MyNodeData::Delay(ref frames) => {
                    // The only nonzero output is slot=0.
                    if edge.from_slot() != 0 {
                        println!("Warning: attempt to read from Delay slot != 0");
                        0f64
                    } else {
                        // t<0 -> value is 0.
                        time.checked_sub(*frames).map_or(0f64, |origin_time| {
                            self.sum_input_to_slot(nodes, node, origin_time, 0, edge.from_ch(), context)
                        })
                    }
                },
                MyNodeData::Constant(ref value) => {
                    // The only nonzero output is slot=0.
                    if edge.from_slot() != 0 {
                        println!("Warning: attempt to read from Constant slot != 0");
                        0f64
                    } else {
                        *value as f64
                    }
                },
                MyNodeData::Multiply => {
                    // The only nonzero output is slot=0.
                    if edge.from_slot() != 0 {
                        println!("Warning: attempt to read from Multiply slot != 0");
                        0f64
                    } else {
                        // Sum all inputs from slot=0 and slot=2 into two separate
                        // variables, then multiply them.
                        let val_a = self.sum_input_to_slot(nodes, node, time, 0, edge.from_ch(), context);
                        let val_b = self.sum_input_to_slot(nodes, node, time, 1, edge.from_ch(), context);
                        val_a * val_b
                    }
                },
                MyNodeData::MultInv => {
                    // The only nonzero output is slot=0.
                    if edge.from_slot() != 0 {
                        println!("Warning: attempt to read from MultInv slot != 0");
                        0f64
                    } else {
                        // Sum all inputs
                        let val_in = self.sum_input_to_slot(nodes, node, time, 0, edge.from_ch(), context);
                        1.0f64 / val_in
                    }
                },
                MyNodeData::ModuloOne => {
                    // The only nonzero output is slot=0.
                    if edge.from_slot() != 0 {
                        println!("Warning: attempt to read from ModOne slot != 0");
                        0f64
                    } else {
                        // Sum all inputs
                        let val_in = self.sum_input_to_slot(nodes, node, time, 0, edge.from_ch(), context);
                        //val_in - val_in.floor()
                        // avoid subtracting unbounded numbers: approximate modulus via this:
                        let rem = val_in.fract();
                        if rem < 0f64 {
                            // TODO: We may be losing precision here, if rem is small.
                            // We should find a way to do true modulus.
                            rem + 1.0f64
                        } else {
                            rem
                        }
                    }
                },
                _ => panic!("Internal RefRenderer error: illegal node type"),
            }
        }
    }
    /// Return the sum of all inputs into a specific slot/channel of the given
    /// node at the given time.
    fn sum_input_to_slot(&self, nodes: &NodeMap, node: &Node, time: u64, slot: u32, ch: u8, context: &Vec<(&NodeMap, NodeHandle)>) -> f64 {
        let edges_in = node.inbound.iter().filter(|in_edge| {
            in_edge.to_slot() == slot && in_edge.to_ch() == ch
        });
        edges_in.map(|edge| self.get_value(nodes, edge, time, context)).sum()
    }

    fn make_node(&self, data: &NodeData) -> MyNodeData {
        match *data {
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
                            "/MultInv" => MyNodeData::MultInv,
                            "/ModuloOne" => MyNodeData::ModuloOne,
                            _ => panic!("Unrecognized primitive effect: {} (full url: {})", url.path(), url),
                        }
                    }
                    None => {
                        let graph = effect.routegraph().as_ref().unwrap();
                        let mut nodes = HashMap::new();
                        for (node, data) in graph.iter_nodes() {
                            nodes.insert(node.clone(), Node::new(self.make_node(data)));
                        }
                        for edge in graph.iter_edges() {
                            nodes.entry(edge.to_full()).or_insert_with(|| {
                                Node::new(MyNodeData::DagIO)
                            }).inbound.insert(edge.clone());
                        }
                        MyNodeData::UserNode(nodes)
                    }
                }
            }
        }
    }
}

impl GraphWatcher for RefRenderer {
    fn on_add_node(&mut self, handle: &NodeHandle, data: &NodeData) {
        let my_node_data = self.make_node(data);
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
