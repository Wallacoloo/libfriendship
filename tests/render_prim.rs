//! Test rendering of primitive effects, through the Dispatch interface.

extern crate libfriendship;
#[macro_use] extern crate ndarray;
extern crate url;

use std::sync::mpsc::{channel, Receiver, Sender};

use ndarray::Array2;
use url::Url;

use libfriendship::{Dispatch, Client};
use libfriendship::dispatch::{OscRouteGraph, OscRenderer};
use libfriendship::render::SparkleRenderer;
use libfriendship::routing::{Edge, EdgeWeight, EffectId, NodeHandle};


struct MyClient {
    /// Where to send the rendered audio.
    tx: Sender<Array2<f32>>,
}
impl Client for MyClient {
    fn audio_rendered(&mut self, buffer: Array2<f32>, _idx: u64) {
        self.tx.send(buffer).unwrap();
    }
}

fn test_setup() -> (Dispatch<SparkleRenderer, MyClient>, Receiver<Array2<f32>>) {
    let (tx, rx) = channel();
    let dispatch = Dispatch::new(SparkleRenderer::default(), MyClient{ tx });
    (dispatch, rx)
}

/// Return the `EffectId` that universally represents `Delay` nodes.
fn delay_id() -> EffectId {
    EffectId::new("Delay".into(), None, vec![Url::parse("primitive:///Delay").unwrap()])
}

/// Return the `EffectId` that universally represents `Sum2` nodes.
fn sum2_id() -> EffectId {
    EffectId::new("Sum2".into(), None, vec![Url::parse("primitive:///Sum2").unwrap()])
}

/// Return the `EffectId` that universally represents `F32Constant` nodes.
fn const_id() -> EffectId {
    EffectId::new("F32Constant".into(), None, vec![Url::parse("primitive:///F32Constant").unwrap()])
}

/// Return the `EffectId` that universally represents `Multiply` nodes.
fn mult_id() -> EffectId {
    EffectId::new("Multiply".into(), None, vec![Url::parse("primitive:///Multiply").unwrap()])
}

/// Return the `EffectId` that universally represents `Divide` nodes.
fn div_id() -> EffectId {
    EffectId::new("Divide".into(), None, vec![Url::parse("primitive:///Divide").unwrap()])
}

/// Return the `EffectId` that universally represents `Modulo` nodes.
fn mod_id() -> EffectId {
    EffectId::new("Modulo".into(), None, vec![Url::parse("primitive:///Modulo").unwrap()])
}

/// Return the `EffectId` that universally represents `Minimum` nodes.
fn min_id() -> EffectId {
    EffectId::new("Minimum".into(), None, vec![Url::parse("primitive:///Minimum").unwrap()])
}

#[test]
fn render_zeros() {
    let (mut dispatch, rx) = test_setup();

    // Read some data from ch=0.
    // This should be all zeros because we have no data being rendered.
    dispatch.dispatch(
        OscRenderer::RenderRange((), (0..4, 1, Default::default()))
    .into()).unwrap();
    let rendered = rx.recv().unwrap();
    assert_eq!(rendered, array![[0f32, 0f32, 0f32, 0f32]]);
}

#[test]
fn render_const() {
    let (mut dispatch, rx) = test_setup();

    // Route a constant into ch=0.
    let handle = NodeHandle::new(1);
    dispatch.dispatch(OscRouteGraph::AddNode( (), (handle, const_id()) ).into()).unwrap();
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new_to_null(handle, EdgeWeight::new((0.5f32).to_bits(), 0)),)).into()).unwrap();
    
    // Read some data from ch=0.
    // This should be all 0.5 because of the new node we added.
    dispatch.dispatch(
        OscRenderer::RenderRange((), (0..4, 1, Default::default()))
    .into()).unwrap();
    let rendered = rx.recv().unwrap();
    assert_eq!(rendered, array![[0.5f32, 0.5f32, 0.5f32, 0.5f32]]);
}

#[test]
fn render_delay() {
    let (mut dispatch, rx) = test_setup();

    // Create delay node (id=1)
    let delay_hnd = NodeHandle::new(1);
    dispatch.dispatch(OscRouteGraph::AddNode( (), (delay_hnd, delay_id()) ).into()).unwrap();
    // Connect delay output to master output.
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new_to_null(delay_hnd, EdgeWeight::new(0, 0)),)).into()).unwrap();

    // Create Constant node (id=2)
    let const_hnd = NodeHandle::new(2);
    dispatch.dispatch(OscRouteGraph::AddNode( (), (const_hnd, const_id()) ).into()).unwrap();
    // Route constant output to delay input
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new(const_hnd, delay_hnd, EdgeWeight::new((0.5f32).to_bits(), 0)),)).into()).unwrap();
    
    // Create Constant node (id=3)
    let const_hnd = NodeHandle::new(3);
    dispatch.dispatch(OscRouteGraph::AddNode( (), (const_hnd, const_id()) ).into()).unwrap();
    // Route constant output to delay AMOUNT
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new(const_hnd, delay_hnd, EdgeWeight::new((2f32).to_bits(), 1)),)).into()).unwrap();

    // Read some data from ch=0.
    // This should be [0, 0, 0.5, 0.5]: constant but delayed by 2.
    dispatch.dispatch(
        OscRenderer::RenderRange((), (0..4, 1, Default::default()))
    .into()).unwrap();
    let rendered = rx.recv().unwrap();
    assert_eq!(rendered, array![[0f32, 0f32, 0.5f32, 0.5f32]]);
}

#[test]
fn render_mult() {
    let (mut dispatch, rx) = test_setup();

    // Create Multiply node (id=1)
    let mult_hnd = NodeHandle::new(1);
    dispatch.dispatch(OscRouteGraph::AddNode( (), (mult_hnd, mult_id()) ).into()).unwrap();
    // Connect delay output to master output.
    dispatch.dispatch(OscRouteGraph::AddEdge(
        (), (Edge::new_to_null(mult_hnd, EdgeWeight::new(0, 0)),)
    ).into()).unwrap();
    
    // Create Constant node (id=2)
    let const_hnd = NodeHandle::new(2);
    dispatch.dispatch(OscRouteGraph::AddNode( (), (const_hnd, const_id()) ).into()).unwrap();
    // Route constant output to multiply input (A)
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new(const_hnd, mult_hnd, EdgeWeight::new((0.5f32).to_bits(), 0)),)).into()).unwrap();
    
    // Create Constant node (id=3)
    let const_hnd = NodeHandle::new(3);
    dispatch.dispatch(OscRouteGraph::AddNode( (), (const_hnd, const_id()) ).into()).unwrap();
    // Route constant output to multiply input (B)
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new(const_hnd, mult_hnd, EdgeWeight::new((-3f32).to_bits(), 1)),)).into()).unwrap();
    
    // Read some data from ch=0.
    // This should be 0.5 * -3.0 = -1.5
    dispatch.dispatch(
        OscRenderer::RenderRange((), (0..4, 1, Default::default()))
    .into()).unwrap();
    let rendered = rx.recv().unwrap();
    assert_eq!(rendered, array![[-1.5f32, -1.5f32, -1.5f32, -1.5f32]]);
}

#[test]
fn render_sum2() {
    let (mut dispatch, rx) = test_setup();

    // Create Multiply node (id=1)
    let sum_hnd = NodeHandle::new(1);
    dispatch.dispatch(OscRouteGraph::AddNode( (), (sum_hnd, sum2_id()) ).into()).unwrap();
    // Connect delay output to master output.
    dispatch.dispatch(OscRouteGraph::AddEdge(
        (), (Edge::new_to_null(sum_hnd, EdgeWeight::new(0, 0)),)
    ).into()).unwrap();
    
    // Create Constant node (id=2)
    let const_hnd = NodeHandle::new(2);
    dispatch.dispatch(OscRouteGraph::AddNode( (), (const_hnd, const_id()) ).into()).unwrap();
    // Route constant output to sum input (A)
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new(const_hnd, sum_hnd, EdgeWeight::new((0.5f32).to_bits(), 0)),)).into()).unwrap();
    
    // Create Constant node (id=3)
    let const_hnd = NodeHandle::new(3);
    dispatch.dispatch(OscRouteGraph::AddNode( (), (const_hnd, const_id()) ).into()).unwrap();
    // Route constant output to sum input (B)
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new(const_hnd, sum_hnd, EdgeWeight::new((-3f32).to_bits(), 1)),)).into()).unwrap();
    
    // Read some data from ch=0.
    // This should be 0.5 + -3.0 = -2.5
    dispatch.dispatch(
        OscRenderer::RenderRange((), (0..4, 1, Default::default()))
    .into()).unwrap();
    let rendered = rx.recv().unwrap();
    assert_eq!(rendered, array![[-2.5f32, -2.5f32, -2.5f32, -2.5f32]]);
}

#[test]
fn render_div() {
    let (mut dispatch, rx) = test_setup();

    // Create Divide node (id=1)
    let div_hnd = NodeHandle::new(1);
    dispatch.dispatch(OscRouteGraph::AddNode( (), (div_hnd, div_id()) ).into()).unwrap();
    // Connect delay output to master output.
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new_to_null(div_hnd, EdgeWeight::new(0, 0)),)).into()).unwrap();
    
    // Create Constant node (id=2)
    let const_hnd = NodeHandle::new(2);
    dispatch.dispatch(OscRouteGraph::AddNode( (), (const_hnd, const_id()) ).into()).unwrap();
    // Route constant output to divide input (A)
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new(const_hnd, div_hnd, EdgeWeight::new((0.5f32).to_bits(), 0)),)).into()).unwrap();
    
    // Create Constant node (id=3)
    let const_hnd = NodeHandle::new(3);
    dispatch.dispatch(OscRouteGraph::AddNode( (), (const_hnd, const_id()) ).into()).unwrap();
    // Route constant output to divide input (B)
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new(const_hnd, div_hnd, EdgeWeight::new((-3f32).to_bits(), 1)),)).into()).unwrap();
    
    // Read some data from ch=0.
    // This should be 0.5 / -3.0 = -0.1666...
    dispatch.dispatch(
        OscRenderer::RenderRange((), (0..4, 1, Default::default()))
    .into()).unwrap();
    let rendered = rx.recv().unwrap();
    let exp = 0.5f32 / -3.0f32;
    assert_eq!(rendered, array![[exp, exp, exp, exp]]);
}

#[test]
fn render_mod() {
    let (mut dispatch, rx) = test_setup();

    // Create Modulo node (id=1)
    let mod_hnd = NodeHandle::new(1);
    dispatch.dispatch(OscRouteGraph::AddNode( (), (mod_hnd, mod_id()) ).into()).unwrap();
    // Connect delay output to master output.
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new_to_null(mod_hnd, EdgeWeight::new(0, 0)),)).into()).unwrap();
    
    // Create Constant node (id=2)
    let const_hnd = NodeHandle::new(2);
    dispatch.dispatch(OscRouteGraph::AddNode( (), (const_hnd, const_id()) ).into()).unwrap();
    // Route constant output to modulo input (A)
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new(const_hnd, mod_hnd, EdgeWeight::new((-3.5f32).to_bits(), 0)),)).into()).unwrap();
    
    // Create Constant node (id=3)
    let const_hnd = NodeHandle::new(3);
    dispatch.dispatch(OscRouteGraph::AddNode( (), (const_hnd, const_id()) ).into()).unwrap();
    // Route constant output to modulo input (B)
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new(const_hnd, mod_hnd, EdgeWeight::new((2f32).to_bits(), 1)),)).into()).unwrap();
    
    // Read some data from ch=0.
    // This should be -3.5 % 2.0 = 0.5
    dispatch.dispatch(
        OscRenderer::RenderRange((), (0..4, 1, Default::default()))
    .into()).unwrap();
    let rendered = rx.recv().unwrap();
    let exp = 0.5f32;
    assert_eq!(rendered, array![[exp, exp, exp, exp]]);
}

#[test]
fn render_min() {
    let (mut dispatch, rx) = test_setup();

    // Create Modulo node (id=1)
    let min_hnd = NodeHandle::new(1);
    dispatch.dispatch(OscRouteGraph::AddNode( (), (min_hnd, min_id()) ).into()).unwrap();
    // Connect delay output to master output.
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new_to_null(min_hnd, EdgeWeight::new(0, 0)),)).into()).unwrap();
    
    // Create Constant node (id=2)
    let const_hnd = NodeHandle::new(2);
    dispatch.dispatch(OscRouteGraph::AddNode( (), (const_hnd, const_id()) ).into()).unwrap();
    // Route constant output to modulo input (A)
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new(const_hnd, min_hnd, EdgeWeight::new((-3.5f32).to_bits(), 0)),)).into()).unwrap();
    
    // Create Constant node (id=3)
    let const_hnd = NodeHandle::new(3);
    dispatch.dispatch(OscRouteGraph::AddNode( (), (const_hnd, const_id()) ).into()).unwrap();
    // Route constant output to modulo input (B)
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new(const_hnd, min_hnd, EdgeWeight::new((2f32).to_bits(), 1)),)).into()).unwrap();
    
    // Read some data from ch=0.
    // This should be min(-3.5, 2.0) = -3.5
    dispatch.dispatch(
        OscRenderer::RenderRange((), (0..4, 1, Default::default()))
    .into()).unwrap();
    let rendered = rx.recv().unwrap();
    let exp = -3.5f32;
    assert_eq!(rendered, array![[exp, exp, exp, exp]]);
}
