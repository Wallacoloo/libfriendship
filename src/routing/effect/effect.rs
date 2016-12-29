extern crate ndarray;

use self::ndarray::Array2;

use routing::routegraph::{RouteEdge, RouteGraph, RouteHalfEdge, RouteNode, RouteNodeHandle};
use super::effectgraph::{EffectGraph, EffectHalfEdge};

/// Describes all information needed to instantiate the effect.
#[derive(Clone, PartialEq, Eq)]
pub struct Effect {
    typename: String,
    /// index [a, b] defines the MINIMUM time elapsed needed for a change in input a
    /// to alter output b; None = infinity
    io_latencies: Array2<Option<u32>>,
}

pub struct RouteImpl {
    inputs: Vec<RouteHalfEdge>,
    // Outputs are assigned by being the Nth input to the right side of the root.
    graph: RouteGraph,
}

pub struct FxGraphImpl {
    inputs: Vec<EffectHalfEdge>,
    graph: EffectGraph,
}

pub enum EffectImpl {
    RouteImpl(RouteImpl),
    FxGraphImpl(FxGraphImpl),
}

impl Effect {
    pub fn new(typename: String, _ch_count: u32) -> Self {
        let n_inputs = 2;
        let n_outputs = 1;
        Effect {
            typename: typename,
            io_latencies: Array2::from_elem((n_inputs, n_outputs), None),
        }
    }
    pub fn n_inputs(&self) -> u32 {
        self.io_latencies.shape()[0] as u32
    }
    pub fn n_outputs(&self) -> u32 {
        self.io_latencies.shape()[1] as u32
    }
    pub fn get_impl(&self) -> EffectImpl {
        // For testing, this effect takes 2 inputs and multiplies them.
        let mut g = RouteGraph::new();
        let root = g.root().clone();
        // Mixer output is the Effect output
        let mixer = g.add_node(RouteNode::new_intermediary());
        g.add_edge(&root, &mixer, RouteEdge::new_right(0));
        let imp = RouteImpl {
            inputs: vec![
                RouteHalfEdge::new(mixer.clone(), RouteEdge::new_left()),
                RouteHalfEdge::new(mixer, RouteEdge::new_right(0)),
            ],
            graph: g,
        };
        EffectImpl::RouteImpl(imp)
    }
    /// Return the minimum causal latency from the given input to the given output
    /// or None if the output is not dependent on the input.
    pub fn min_latency(&self, in_idx: u32, out_idx: u32) -> Option<u32> {
        self.io_latencies[[in_idx as usize, out_idx as usize]]
    }
}

impl Default for Effect {
    /// Returns a "passthrough" effect, with 0 channels (i.e. a NOP)
    fn default() -> Self {
        Effect::new("passthrough".to_string(), 0)
    }
}

