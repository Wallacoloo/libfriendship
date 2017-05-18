use std::collections::{HashMap, HashSet};

use render::Renderer;
use resman::AudioBuffer;
use routing::{DagHandle, Edge, GraphWatcher, NodeData, NodeHandle};
use routing::effect::{PrimitiveEffect, EffectData};
use util::unpack_f32;

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
    /// Primitive effect (delay, constant, etc).
    Primitive(PrimitiveEffect),
    /// External audio.
    Buffer(AudioBuffer),
    /// This node is a DAG definition. i.e. it holds the output edges of a DAG.
    DagIO,
}

impl Renderer for RefRenderer {
    fn get_sample(&mut self, time: u64, slot: u32) -> f32 {
        // Try to find the edge that goes to -> (Null, slot==slot)
        let root_handle = NodeHandle::toplevel();
        // empty graph is 0 = silence
        self.nodes.get(&root_handle).map_or(0f32, |node| {
            // find all edges to ([Null], slot=slot)
            self.sum_input_to_slot(&self.nodes, node, time, slot, &Vec::new()) as f32
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
            // Sum the inputs to the matching slot
            self.sum_input_to_slot(new_nodes, &new_nodes[&head], time, edge.from_slot(), &new_context)
        } else {
            // Reading from another node within the DAG
            let node = &nodes[&from];
            match node.data {
                MyNodeData::UserNode(ref new_nodes) => {
                    let mut new_context = context.clone();
                    new_context.push((nodes, from));
                    // Now find the *output* of the sub dag (or 0 if the sub dag has no outputs)
                    new_nodes.get(&NodeHandle::toplevel()).map_or(0f64, |root_node| {
                        self.sum_input_to_slot(new_nodes, root_node, time, edge.from_slot(), &new_context)
                    })
                },
                MyNodeData::Primitive(prim) => match prim {
                    // Output = sum of all inputs to slot 0.
                    PrimitiveEffect::Delay => {
                        // The only nonzero output is slot=0.
                        if edge.from_slot() != 0 {
                            println!("Warning: attempt to read from Delay slot != 0");
                            0f64
                        } else {
                            let delay_frames = self.sum_input_to_slot(nodes, node, time, 1, context);
                            // Clamp delay value to [0, u64::max]
                            let delay_int = if delay_frames < 0f64 {
                                0u64
                            } else if delay_frames > u64::max_value() as f64 {
                                // TODO: u64::max isn't precisely representable in f64;
                                // will this cause issues?
                                u64::max_value()
                            } else {
                                // Note: this conversion is flooring.
                                delay_frames as u64
                            };
                            // t<0 -> value is 0.
                            time.checked_sub(delay_int).map_or(0f64, |origin_time| {
                                self.sum_input_to_slot(nodes, node, origin_time, 0, context)
                            })
                        }
                    },
                    PrimitiveEffect::F32Constant => {
                        // Float value is encoded via the slot.
                        let value = edge.from_slot();
                        unpack_f32(value) as f64
                    },
                    PrimitiveEffect::Multiply => {
                        // The only nonzero output is slot=0.
                        if edge.from_slot() != 0 {
                            println!("Warning: attempt to read from Multiply slot != 0");
                            0f64
                        } else {
                            // Sum all inputs from slot=0 and slot=2 into two separate
                            // variables, then multiply them.
                            let val_a = self.sum_input_to_slot(nodes, node, time, 0, context);
                            let val_b = self.sum_input_to_slot(nodes, node, time, 1, context);
                            val_a * val_b
                        }
                    },
                    PrimitiveEffect::Divide => {
                        // The only nonzero output is slot=0.
                        if edge.from_slot() != 0 {
                            println!("Warning: attempt to read from MultInv slot != 0");
                            0f64
                        } else {
                            // Sum all inputs
                            let dividend = self.sum_input_to_slot(nodes, node, time, 0, context);
                            let divisor = self.sum_input_to_slot(nodes, node, time, 1, context);
                            dividend / divisor
                        }
                    },
                    PrimitiveEffect::Minimum => {
                        // The only nonzero output is slot=0.
                        if edge.from_slot() != 0 {
                            println!("Warning: attempt to read from Modulo slot != 0");
                            0f64
                        } else {
                            let input_a = self.sum_input_to_slot(nodes, node, time, 0, context);
                            let input_b = self.sum_input_to_slot(nodes, node, time, 1, context);
                            input_a.min(input_b)
                        }
                    }
                    PrimitiveEffect::Modulo => {
                        // The only nonzero output is slot=0.
                        if edge.from_slot() != 0 {
                            println!("Warning: attempt to read from Modulo slot != 0");
                            0f64
                        } else {
                            // Sum all dividends
                            let dividend = self.sum_input_to_slot(nodes, node, time, 0, context);
                            // Sum all divisors
                            let divisor = self.sum_input_to_slot(nodes, node, time, 1, context);
                            let rem = dividend % divisor;
                            if rem < 0f64 {
                                // TODO: We may be losing precision here, if rem is small.
                                // We should find a way to do true modulus.
                                rem + divisor
                            } else {
                                rem
                            }
                        }
                    },
                },
                _ => panic!("Internal RefRenderer error: illegal node type"),
            }
        }
    }
    /// Return the sum of all inputs into a specific slot of the given
    /// node at the given time.
    fn sum_input_to_slot(&self, nodes: &NodeMap, node: &Node, time: u64, slot: u32, context: &Vec<(&NodeMap, NodeHandle)>) -> f64 {
        let edges_in = node.inbound.iter().filter(|in_edge| {
            in_edge.to_slot() == slot
        });
        edges_in.map(|edge| self.get_value(nodes, edge, time, context)).sum()
    }

    fn make_node(&self, data: &NodeData) -> MyNodeData {
        match *data {
            NodeData::Effect(ref effect) => {
                match *effect.data() {
                    EffectData::Primitive(e) => MyNodeData::Primitive(e),
                    EffectData::Buffer(ref buff) => MyNodeData::Buffer(buff.clone()),
                    EffectData::RouteGraph(ref graph) => {
                        let mut nodes = HashMap::new();
                        for (node, data) in graph.iter_nodes() {
                            nodes.insert(*node, Node::new(self.make_node(data)));
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
        self.nodes.insert(*handle, Node::new(my_node_data));
        // If the node is part of a new DAG, allocate data so that future edges
        // to null within the DAG can be held.
        self.nodes.entry(NodeHandle::new_dag(*handle.dag_handle())).or_insert_with(|| {
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
