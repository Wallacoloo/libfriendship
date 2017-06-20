use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use ndarray::Array2;
use jagged_array::Jagged2;

use render::Renderer;
use resman::AudioBuffer;
use routing::{Edge, GraphWatcher, NodeData, NodeHandle};
use routing::effect::{PrimitiveEffect, EffectData};
use streaming_iterator::StreamingIterator;
use util::unpack_f32;


#[derive(Debug, Default)]
struct NodeMap {
    nodes: HashMap<NodeHandle, Node>,
    output_edges: Vec<Option<Edge>>,
}

#[derive(Default, Debug)]
pub struct RefRenderer {
    nodes: NodeMap,
    /// inputs[slot][sample] represents the value of the external input at
    /// the given slot and time. On OOB, it is assumed to be 0.
    inputs: Vec<Vec<f32>>,
    /// Next expected sample to be queried.
    /// This is tracked because if we do a seek, the inputs need to be zero'd.
    head: u64,
}

#[derive(Debug)]
struct Node {
    data: MyNodeData,
    /// Inbound edges, indexed by slot idx.
    inbound: Vec<Option<Edge>>,
}

#[derive(Debug)]
enum MyNodeData {
    /// This node is a non-primitive effect.
    UserNode(NodeMap),
    /// Primitive effect (delay, constant, etc).
    Primitive(PrimitiveEffect),
    /// External audio.
    Buffer(AudioBuffer),
}

impl Renderer for RefRenderer {
    fn fill_buffer(&mut self, buff: &mut Array2<f32>, idx: u64, inputs: Jagged2<f32>) {
        let (n_slots, n_times) = buff.dim().into();
        // Store inputs for future use
        {
            // If this is a seek operation, forget historical input values.
            if idx != self.head {
                for slot in &mut self.inputs {
                    *slot = Vec::new();
                    // NB: Improper indexing on 32-bit OS, but will OOM first.
                    slot.resize(idx as usize, 0f32);
                }
            }
            // Make sure we have storage space for all inputs.
            while self.inputs.len() < buff.len() {
                // NB: Improper indexing on 32-bit OS, but will OOM first.
                let mut v = Vec::with_capacity(idx as usize);
                v.resize(idx as usize, 0f32);
                self.inputs.push(v);
            }
            let mut stream = inputs.stream();
            let mut self_it = self.inputs.iter_mut();
            while let (Some(row), Some(vec_dest)) = (stream.next(), self_it.next()) {
                assert_eq!(vec_dest.len(), idx as usize);
                vec_dest.extend(row);
                assert!(vec_dest.len() <= idx as usize + n_times); // cannot send inputs ahead of outputs.
                let pad_val = vec_dest.last().cloned().unwrap_or(0f32);
                vec_dest.resize(idx as usize +n_times, pad_val);
            }
        }

        // Calculate outputs
        for slot in 0..n_slots as u32 {
            for time in idx..idx+n_times as u64 {
                buff[[slot as usize, (time - idx) as usize]] = self.get_sample(time, slot);
            }
        }
        // Keep track of the playhead
        self.head = idx + n_times as u64;
    }
}

impl RefRenderer {
    /// Get the output at a particular time and to a particular output slot.
    fn get_sample(&mut self, time: u64, slot: u32) -> f32 {
        self.nodes.get_output(time, slot, |time2, slot2| {
            *self.inputs.get(slot2 as usize)
                .and_then(|v| v.get(time2 as usize))
                .unwrap_or(&0f32)
        })
    }
    /// Allocate renderer data based on data from a RouteGraph node.
    fn make_node(&self, effect: &NodeData) -> MyNodeData {
        match *effect.data() {
            EffectData::Primitive(e) => MyNodeData::Primitive(e),
            EffectData::Buffer(ref buff) => MyNodeData::Buffer(buff.clone()),
            EffectData::RouteGraph(ref graph) => {
                let mut nodes: NodeMap = Default::default();

                for (node, data) in graph.iter_nodes() {
                    nodes.insert(*node, Node::new(self.make_node(data)));
                }
                for edge in graph.iter_edges() {
                    nodes.add_edge(edge);
                }
                MyNodeData::UserNode(nodes)
            }
        }
    }
}

impl GraphWatcher for RefRenderer {
    fn on_add_node(&mut self, handle: &NodeHandle, data: &NodeData) {
        let my_node_data = self.make_node(data);
        self.nodes.insert(*handle, Node::new(my_node_data));
    }
    fn on_del_node(&mut self, handle: &NodeHandle) {
        self.nodes.remove(handle);
    }
    fn on_add_edge(&mut self, edge: &Edge) {
        self.nodes.add_edge(edge);
    }
    fn on_del_edge(&mut self, edge: &Edge) {
        let inbound = if edge.to_full().is_toplevel() {
            &mut self.nodes.output_edges
        } else {
            &mut self.nodes.get_mut(&edge.to_full()).expect("Attempt to delete edge, but it was never created!").inbound
        };
        if let Some(stored_edge) = inbound.get_mut(edge.to_slot() as usize) {
            *stored_edge = None
        }
    }
}

impl NodeMap {
    /// Add an edge that connects two nodes within this graph.
    fn add_edge(&mut self, edge: &Edge) {
        let inbound = if edge.to_full().is_toplevel() {
            &mut self.output_edges
        } else {
            &mut self.nodes.get_mut(&edge.to_full()).unwrap().inbound
        };
        let slot: u32 = edge.to_slot().into();
        let slot = slot as usize;
        // allocate space to store the edge.
        if inbound.len() <= slot { inbound.resize(slot+1, None) }

        inbound[slot] = Some(edge.clone());
    }
    /// Get the value of a particular output slot at a given time.
    /// This will recurse down, all the way to the input to this node itself.
    /// `get_input(time, slot)` will be called (multiple times, with different args)
    /// in order to query whatever is input to this node.
    fn get_output<F: Fn(u64, u32) -> f32>(&self, time: u64, slot: u32, get_input: F) -> f32 {
        let out_edge = self.output_edges.get(slot as usize);
        self.get_maybe_edge_value(time, out_edge, &get_input)
    }
    /// Wrapper around `get_edge_value` that will return 0f32 if maybe_edge is not
    /// `Some(&Some(edge))`.
    fn get_maybe_edge_value(&self, time: u64,
        maybe_edge: Option<&Option<Edge>>, get_input: &Fn(u64, u32) -> f32) -> f32
    {
        if let Some(&Some(ref edge)) = maybe_edge {
            self.get_edge_value(time, &edge, get_input)
        } else {
            // Edge doesn't exist; value is zero.
            0f32
        }
    }
    /// Get the value on an edge at a specific time.
    /// This will recurse down, all the way to the input to this node itself.
    /// `get_input(time, slot)` will be called (multiple times, with different args)
    /// in order to query whatever is input to this node.
    fn get_edge_value(&self, time: u64, edge: &Edge, get_input: &Fn(u64, u32) -> f32) -> f32 {
        let from = edge.from_full();
        let from_slot = edge.from_slot();
        if *from.node_handle() == None {
            // reading from an input
            get_input(time, from_slot)
        } else {
            // Reading from another node within the DAG
            let node = &self.nodes[&from];
            match node.data {
                MyNodeData::UserNode(ref new_nodes) => {
                    new_nodes.get_output(time, from_slot, |time2, slot2| {
                        // get the input to this node.
                        let in_edge = node.inbound.get(slot2 as usize);
                        self.get_maybe_edge_value(time2, in_edge, get_input)
                    })
                },
                MyNodeData::Primitive(prim) => match prim {
                    // Output = sum of all inputs to slot 0.
                    PrimitiveEffect::Delay => {
                        // The only nonzero output is slot=0.
                        if from_slot != 0 {
                            warn!("Attempt to read from Delay slot != 0");
                            0f32
                        } else {
                            let delay_frames = self.get_maybe_edge_value(time, node.inbound.get(1), get_input);
                            // Clamp delay value to [0, u64::max]
                            let delay_int = if delay_frames < 0f32 {
                                0u64
                            } else if delay_frames > u64::max_value() as f32 {
                                // TODO: u64::max isn't precisely representable in f32;
                                // will this cause issues?
                                // TODO: this is technically incorrect when time=u64::max_value,
                                // as this results in returning the value at t=0.
                                u64::max_value()
                            } else {
                                // Note: this conversion is flooring.
                                delay_frames as u64
                            };
                            // t<0 -> value is 0.
                            time.checked_sub(delay_int).map_or(0f32, |origin_time| {
                                self.get_maybe_edge_value(origin_time, node.inbound.get(0), get_input)
                            })
                        }
                    },
                    PrimitiveEffect::F32Constant => {
                        // Float value is encoded via the slot.
                        unpack_f32(from_slot)
                    },
                    PrimitiveEffect::Multiply => {
                        // The only nonzero output is slot=0.
                        if from_slot != 0 {
                            warn!("Attempt to read from Multiply slot != 0");
                            0f32
                        } else {
                            let input_left = self.get_maybe_edge_value(time, node.inbound.get(0), get_input);
                            let input_right = self.get_maybe_edge_value(time, node.inbound.get(1), get_input);
                            input_left * input_right
                        }
                    },
                    PrimitiveEffect::Sum2 => {
                        // The only nonzero output is slot=0.
                        if from_slot != 0 {
                            warn!("Attempt to read from Sum2 slot != 0");
                            0f32
                        } else {
                            let input_left = self.get_maybe_edge_value(time, node.inbound.get(0), get_input);
                            let input_right = self.get_maybe_edge_value(time, node.inbound.get(1), get_input);
                            input_left + input_right
                        }
                    },
                    PrimitiveEffect::Divide => {
                        // The only nonzero output is slot=0.
                        if from_slot != 0 {
                            warn!("Attempt to read from Divide slot != 0");
                            0f32
                        } else {
                            let dividend = self.get_maybe_edge_value(time, node.inbound.get(0), get_input);
                            let divisor = self.get_maybe_edge_value(time, node.inbound.get(1), get_input);
                            dividend / divisor
                        }
                    },
                    PrimitiveEffect::Minimum => {
                        // The only nonzero output is slot=0.
                        if from_slot != 0 {
                            warn!("Attempt to read from Minimum slot != 0");
                            0f32
                        } else {
                            let input_left = self.get_maybe_edge_value(time, node.inbound.get(0), get_input);
                            let input_right = self.get_maybe_edge_value(time, node.inbound.get(1), get_input);
                            input_left.min(input_right)
                        }
                    }
                    PrimitiveEffect::Modulo => {
                        // The only nonzero output is slot=0.
                        if from_slot != 0 {
                            warn!("Attempt to read from Modulo slot != 0");
                            0f32
                        } else {
                            let dividend = self.get_maybe_edge_value(time, node.inbound.get(0), get_input);
                            let divisor = self.get_maybe_edge_value(time, node.inbound.get(1), get_input);
                            let rem = dividend % divisor;
                            if rem < 0f32 {
                                // TODO: We may be losing precision here, if rem is small.
                                // We should find a way to do true modulus.
                                rem + divisor
                            } else {
                                rem
                            }
                        }
                    },
                },
                MyNodeData::Buffer(ref buf) => buf.get(time, from_slot),
            }
        }
    }
}


impl Node {
    fn new(data: MyNodeData) -> Self {
        Node {
            data: data,
            inbound: Vec::new(),
        }
    }
}


impl Deref for NodeMap {
    type Target = HashMap<NodeHandle, Node>;
    fn deref(&self) -> &Self::Target {
        &self.nodes
    }
}

impl DerefMut for NodeMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.nodes
    }
}
