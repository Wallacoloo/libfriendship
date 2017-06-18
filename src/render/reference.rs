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


#[derive(Debug)]
struct NodeMap {
    nodes: HashMap<NodeHandle, Node>,
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

#[derive(Default, Debug)]
struct Node {
    data: Option<MyNodeData>,
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
    fn get_sample(&mut self, time: u64, slot: u32) -> f32 {
        let node = &self.nodes[&NodeHandle::toplevel()];
        self.get_input_to_slot(&self.nodes, &node, time, slot, &Vec::new()) as f32
    }
    /// Get the value on an edge at a particular time
    /// When backtracking from the output, we push each Node onto the context if we enter inside of
    ///   it (i.e. if it's a nested DAG) & pop when exiting.
    fn get_value(&self, nodes: &NodeMap, edge: &Edge, time: u64, context: &Vec<(&NodeMap, NodeHandle)>) -> f64 {
        let from = edge.from_full();
        if *from.node_handle() == None {
            if context.is_empty() {
                // toplevel input (i.e. external input)
                let slot = edge.from_slot() as usize;
                let value = *self.inputs.get(slot).and_then(|v| v.get(time as usize)).unwrap_or(&0f32) as f64;
                trace!("Renderer reading from ext input [t={}] [slot={}]: {}", time, slot, value);
                value
            } else {
                // Reading from one of the inputs to the top of `context`
                // TODO: we can avoid cloning by reversing the pop after recursing.
                let mut new_context = context.clone();
                let (new_nodes, head) = new_context.pop().unwrap();
                // Get the input to the matching slot
                self.get_input_to_slot(new_nodes, &new_nodes[&head], time, edge.from_slot(), &new_context)
            }
        } else {
            // Reading from another node within the DAG
            let node = &nodes[&from];
            match *node.data.as_ref().expect("Expected node to have associated data") {
                MyNodeData::UserNode(ref new_nodes) => {
                    let mut new_context = context.clone();
                    new_context.push((nodes, from));
                    // Now find the *output* of the sub dag (or 0 if the sub dag has no outputs)
                    new_nodes.get(&NodeHandle::toplevel()).map_or(0f64, |root_node| {
                        self.get_input_to_slot(new_nodes, root_node, time, edge.from_slot(), &new_context)
                    })
                },
                MyNodeData::Primitive(prim) => match prim {
                    // Output = sum of all inputs to slot 0.
                    PrimitiveEffect::Delay => {
                        // The only nonzero output is slot=0.
                        if edge.from_slot() != 0 {
                            warn!("Attempt to read from Delay slot != 0");
                            0f64
                        } else {
                            let delay_frames = self.get_input_to_slot(nodes, node, time, 1, context);
                            // Clamp delay value to [0, u64::max]
                            let delay_int = if delay_frames < 0f64 {
                                0u64
                            } else if delay_frames > u64::max_value() as f64 {
                                // TODO: u64::max isn't precisely representable in f64;
                                // will this cause issues?
                                // TODO: this is technically incorrect when time=u64::max_value,
                                // as this results in returning the value at t=0.
                                u64::max_value()
                            } else {
                                // Note: this conversion is flooring.
                                delay_frames as u64
                            };
                            // t<0 -> value is 0.
                            time.checked_sub(delay_int).map_or(0f64, |origin_time| {
                                self.get_input_to_slot(nodes, node, origin_time, 0, context)
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
                            warn!("Attempt to read from Multiply slot != 0");
                            0f64
                        } else {
                            let val_a = self.get_input_to_slot(nodes, node, time, 0, context);
                            let val_b = self.get_input_to_slot(nodes, node, time, 1, context);
                            val_a * val_b
                        }
                    },
                    PrimitiveEffect::Sum2 => {
                        // The only nonzero output is slot=0.
                        if edge.from_slot() != 0 {
                            warn!("Attempt to read from Sum2 slot != 0");
                            0f64
                        } else {
                            let input_left = self.get_input_to_slot(nodes, node, time, 0, context);
                            let input_right = self.get_input_to_slot(nodes, node, time, 1, context);
                            input_left + input_right
                        }
                    },
                    PrimitiveEffect::Divide => {
                        // The only nonzero output is slot=0.
                        if edge.from_slot() != 0 {
                            warn!("Attempt to read from Divide slot != 0");
                            0f64
                        } else {
                            let dividend = self.get_input_to_slot(nodes, node, time, 0, context);
                            let divisor = self.get_input_to_slot(nodes, node, time, 1, context);
                            dividend / divisor
                        }
                    },
                    PrimitiveEffect::Minimum => {
                        // The only nonzero output is slot=0.
                        if edge.from_slot() != 0 {
                            warn!("Attempt to read from Minimum slot != 0");
                            0f64
                        } else {
                            let input_a = self.get_input_to_slot(nodes, node, time, 0, context);
                            let input_b = self.get_input_to_slot(nodes, node, time, 1, context);
                            input_a.min(input_b)
                        }
                    }
                    PrimitiveEffect::Modulo => {
                        // The only nonzero output is slot=0.
                        if edge.from_slot() != 0 {
                            warn!("Attempt to read from Modulo slot != 0");
                            0f64
                        } else {
                            let dividend = self.get_input_to_slot(nodes, node, time, 0, context);
                            let divisor = self.get_input_to_slot(nodes, node, time, 1, context);
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
                MyNodeData::Buffer(ref buf) => buf.get(time, edge.from_slot()) as f64,
            }
        }
    }
    /// Return the input into a specific slot of the given
    /// node at the given time.
    fn get_input_to_slot(&self, nodes: &NodeMap, node: &Node, time: u64, slot: u32, context: &Vec<(&NodeMap, NodeHandle)>) -> f64 {
        if let Some(&Some(ref edge)) = node.inbound.get(slot as usize) {
            self.get_value(nodes, &edge, time, context)
        } else {
            0f64
        }
    }

    fn make_node(&self, effect: &NodeData) -> MyNodeData {
        match *effect.data() {
            EffectData::Primitive(e) => MyNodeData::Primitive(e),
            EffectData::Buffer(ref buff) => MyNodeData::Buffer(buff.clone()),
            EffectData::RouteGraph(ref graph) => {
                let mut nodes: NodeMap = Default::default();

                for (node, data) in graph.iter_nodes() {
                    nodes.insert(*node, Node::new(Some(self.make_node(data))));
                }
                for edge in graph.iter_edges() {
                    RefRenderer::helper_add_edge(&mut nodes, edge);
                }
                MyNodeData::UserNode(nodes)
            }
        }
    }
    fn helper_add_edge(nodes: &mut NodeMap, edge: &Edge) {
        let inbound = &mut nodes.get_mut(&edge.to_full()).unwrap().inbound;
        let slot: u32 = edge.to_slot().into();
        let slot = slot as usize;
        // allocate space to store the edge.
        if inbound.len() <= slot { inbound.resize(slot+1, None) }

        inbound[slot] = Some(edge.clone());
    }
}

impl GraphWatcher for RefRenderer {
    fn on_add_node(&mut self, handle: &NodeHandle, data: &NodeData) {
        let my_node_data = self.make_node(data);
        self.nodes.insert(*handle, Node::new(Some(my_node_data)));
    }
    fn on_del_node(&mut self, handle: &NodeHandle) {
        self.nodes.remove(handle);
    }
    fn on_add_edge(&mut self, edge: &Edge) {
        RefRenderer::helper_add_edge(&mut self.nodes, edge);
    }
    fn on_del_edge(&mut self, edge: &Edge) {
        let inbound = &mut self.nodes.get_mut(&edge.to_full()).unwrap().inbound;
        if let Some(stored_edge) = inbound.get_mut(edge.to_slot() as usize) {
            *stored_edge = None
        }
    }
}


impl Node {
    fn new(data: Option<MyNodeData>) -> Self {
        Node {
            data: data,
            inbound: Vec::new(),
        }
    }
}


impl Default for NodeMap {
    fn default() -> Self {
        // Create a NodeMap that already has an entry for the toplevel so that it can receive
        // edges.
        let nodes = Some((NodeHandle::toplevel(), Default::default())).into_iter().collect();
        Self { nodes }
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
