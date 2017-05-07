//! Test rendering of primitive effects, through the Dispatch interface.

extern crate libfriendship;
extern crate url;

use std::sync::mpsc::{channel, Receiver, Sender};

use url::Url;

use libfriendship::{Dispatch, Client};
use libfriendship::dispatch::{OscRouteGraph, OscRenderer};
use libfriendship::render::RefRenderer;
use libfriendship::routing::{adjlist, DagHandle, Edge, EdgeWeight, EffectMeta, NodeHandle};
use libfriendship::util::pack_f32;


struct MyClient {
    /// Where to send the rendered audio.
    tx: Sender<Vec<f32>>,
}
impl Client for MyClient {
    fn audio_rendered(&mut self, buffer: &[f32], _idx: u64, _num_ch: u8) {
        self.tx.send(buffer.iter().cloned().collect()).unwrap();
    }
}

fn test_setup() -> (Dispatch<RefRenderer>, Receiver<Vec<f32>>) {
    let (tx, rx) = channel();
    let client = Box::new(MyClient{ tx });
    let mut dispatch: Dispatch<RefRenderer> = Dispatch::new();
    dispatch.register_client(client);
    (dispatch, rx)
}

/// Return the EffectMeta that universally represents Delay nodes.
fn delay_meta() -> EffectMeta {
    EffectMeta::new("Delay".to_string(), None, [Url::parse("primitive:///Delay").unwrap()].iter().cloned())
}

/// Return the EffectMeta that universally represents F32Constant nodes.
fn const_meta() -> EffectMeta {
    EffectMeta::new("F32Constant".to_string(), None, [Url::parse("primitive:///F32Constant").unwrap()].iter().cloned())
}

/// Return the EffectMeta that universally represents Multiply nodes.
fn mult_meta() -> EffectMeta {
    EffectMeta::new("Multiply".to_string(), None, [Url::parse("primitive:///Multiply").unwrap()].iter().cloned())
}

/// Return the EffectMeta that universally represents Divide nodes.
fn div_meta() -> EffectMeta {
    EffectMeta::new("Divide".to_string(), None, [Url::parse("primitive:///Divide").unwrap()].iter().cloned())
}

/// Return the EffectMeta that universally represents Modulo nodes.
fn mod_meta() -> EffectMeta {
    EffectMeta::new("Modulo".to_string(), None, [Url::parse("primitive:///Modulo").unwrap()].iter().cloned())
}

/// Return the EffectMeta that universally represents Min nodes.
fn min_meta() -> EffectMeta {
    EffectMeta::new("Minimum".to_string(), None, [Url::parse("primitive:///Minimum").unwrap()].iter().cloned())
}

#[test]
fn render_zeros() {
    let (mut dispatch, rx) = test_setup();

    // Read some data from ch=0.
    // This should be all zeros because we have no data being rendered.
    dispatch.dispatch(
        OscRenderer::RenderRange((), (0, 4, 1))
    .into()).unwrap();
    let rendered = rx.recv().unwrap();
    assert_eq!(rendered, vec![0f32, 0f32, 0f32, 0f32]);
}

#[test]
fn render_const() {
    let (mut dispatch, rx) = test_setup();

    // Route a constant into ch=0.
    let handle = NodeHandle::new_node(DagHandle::toplevel(), 1);
    dispatch.dispatch(OscRouteGraph::AddNode((), (handle, adjlist::NodeData::Effect(
        const_meta()
    ))).into()).unwrap();
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new_to_null(handle, EdgeWeight::new(pack_f32(0.5f32), 0, 0, 0)),)).into()).unwrap();
    
    // Read some data from ch=0.
    // This should be all 0.5 because of the new node we added.
    dispatch.dispatch(
        OscRenderer::RenderRange((), (0, 4, 1))
    .into()).unwrap();
    let rendered = rx.recv().unwrap();
    assert_eq!(rendered, vec![0.5f32, 0.5f32, 0.5f32, 0.5f32]);
}

#[test]
fn render_delay() {
    let (mut dispatch, rx) = test_setup();

    // Create delay node (id=1)
    let delay_hnd = NodeHandle::new_node(DagHandle::toplevel(), 1);
    dispatch.dispatch(OscRouteGraph::AddNode((), (delay_hnd, adjlist::NodeData::Effect(
        delay_meta()
    ))).into()).unwrap();
    // Connect delay output to master output.
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new_to_null(delay_hnd, EdgeWeight::new(0, 0, 0, 0)),)).into()).unwrap();

    // Create Constant node (id=2)
    let const_hnd = NodeHandle::new_node(DagHandle::toplevel(), 2);
    dispatch.dispatch(OscRouteGraph::AddNode((), (const_hnd, adjlist::NodeData::Effect(
        const_meta()
    ))).into()).unwrap();
    // Route constant output to delay input
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new(const_hnd, delay_hnd, EdgeWeight::new(pack_f32(0.5f32), 0, 0, 0)).unwrap(),)).into()).unwrap();
    
    // Create Constant node (id=3)
    let const_hnd = NodeHandle::new_node(DagHandle::toplevel(), 3);
    dispatch.dispatch(OscRouteGraph::AddNode((), (const_hnd, adjlist::NodeData::Effect(
        const_meta()
    ))).into()).unwrap();
    // Route constant output to delay AMOUNT
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new(const_hnd, delay_hnd, EdgeWeight::new(pack_f32(2f32), 0, 1, 0)).unwrap(),)).into()).unwrap();

    // Read some data from ch=0.
    // This should be [0, 0, 0.5, 0.5]: constant but delayed by 2.
    dispatch.dispatch(
        OscRenderer::RenderRange((), (0, 4, 1))
    .into()).unwrap();
    let rendered = rx.recv().unwrap();
    assert_eq!(rendered, vec![0f32, 0f32, 0.5f32, 0.5f32]);
}

#[test]
fn render_mult() {
    let (mut dispatch, rx) = test_setup();

    // Create Multiply node (id=1)
    let mult_hnd = NodeHandle::new_node(DagHandle::toplevel(), 1);
    dispatch.dispatch(OscRouteGraph::AddNode((), (mult_hnd, adjlist::NodeData::Effect(
        mult_meta()
    ))).into()).unwrap();
    // Connect delay output to master output.
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new_to_null(mult_hnd, EdgeWeight::new(0, 0, 0, 0)),)).into()).unwrap();
    
    // Create Constant node (id=2)
    let const_hnd = NodeHandle::new_node(DagHandle::toplevel(), 2);
    dispatch.dispatch(OscRouteGraph::AddNode((), (const_hnd, adjlist::NodeData::Effect(
        const_meta()
    ))).into()).unwrap();
    // Route constant output to multiply input (A)
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new(const_hnd, mult_hnd, EdgeWeight::new(pack_f32(0.5f32), 0, 0, 0)).unwrap(),)).into()).unwrap();
    
    // Create Constant node (id=3)
    let const_hnd = NodeHandle::new_node(DagHandle::toplevel(), 3);
    dispatch.dispatch(OscRouteGraph::AddNode((), (const_hnd, adjlist::NodeData::Effect(
        const_meta()
    ))).into()).unwrap();
    // Route constant output to multiply input (B)
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new(const_hnd, mult_hnd, EdgeWeight::new(pack_f32(-3f32), 0, 1, 0)).unwrap(),)).into()).unwrap();
    
    // Read some data from ch=0.
    // This should be 0.5 * -3.0 = -1.5
    dispatch.dispatch(
        OscRenderer::RenderRange((), (0, 4, 1))
    .into()).unwrap();
    let rendered = rx.recv().unwrap();
    assert_eq!(rendered, vec![-1.5f32, -1.5f32, -1.5f32, -1.5f32]);
}

#[test]
fn render_div() {
    let (mut dispatch, rx) = test_setup();

    // Create Divide node (id=1)
    let div_hnd = NodeHandle::new_node(DagHandle::toplevel(), 1);
    dispatch.dispatch(OscRouteGraph::AddNode((), (div_hnd, adjlist::NodeData::Effect(
        div_meta()
    ))).into()).unwrap();
    // Connect delay output to master output.
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new_to_null(div_hnd, EdgeWeight::new(0, 0, 0, 0)),)).into()).unwrap();
    
    // Create Constant node (id=2)
    let const_hnd = NodeHandle::new_node(DagHandle::toplevel(), 2);
    dispatch.dispatch(OscRouteGraph::AddNode((), (const_hnd, adjlist::NodeData::Effect(
        const_meta()
    ))).into()).unwrap();
    // Route constant output to divide input (A)
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new(const_hnd, div_hnd, EdgeWeight::new(pack_f32(0.5f32), 0, 0, 0)).unwrap(),)).into()).unwrap();
    
    // Create Constant node (id=3)
    let const_hnd = NodeHandle::new_node(DagHandle::toplevel(), 3);
    dispatch.dispatch(OscRouteGraph::AddNode((), (const_hnd, adjlist::NodeData::Effect(
        const_meta()
    ))).into()).unwrap();
    // Route constant output to divide input (B)
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new(const_hnd, div_hnd, EdgeWeight::new(pack_f32(-3f32), 0, 1, 0)).unwrap(),)).into()).unwrap();
    
    // Read some data from ch=0.
    // This should be 0.5 / -3.0 = -0.1666...
    dispatch.dispatch(
        OscRenderer::RenderRange((), (0, 4, 1))
    .into()).unwrap();
    let rendered = rx.recv().unwrap();
    let exp = 0.5f32 / -3.0f32;
    assert_eq!(rendered, vec![exp, exp, exp, exp]);
}

#[test]
fn render_mod() {
    let (mut dispatch, rx) = test_setup();

    // Create Modulo node (id=1)
    let mod_hnd = NodeHandle::new_node(DagHandle::toplevel(), 1);
    dispatch.dispatch(OscRouteGraph::AddNode((), (mod_hnd, adjlist::NodeData::Effect(
        mod_meta()
    ))).into()).unwrap();
    // Connect delay output to master output.
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new_to_null(mod_hnd, EdgeWeight::new(0, 0, 0, 0)),)).into()).unwrap();
    
    // Create Constant node (id=2)
    let const_hnd = NodeHandle::new_node(DagHandle::toplevel(), 2);
    dispatch.dispatch(OscRouteGraph::AddNode((), (const_hnd, adjlist::NodeData::Effect(
        const_meta()
    ))).into()).unwrap();
    // Route constant output to modulo input (A)
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new(const_hnd, mod_hnd, EdgeWeight::new(pack_f32(-3.5f32), 0, 0, 0)).unwrap(),)).into()).unwrap();
    
    // Create Constant node (id=3)
    let const_hnd = NodeHandle::new_node(DagHandle::toplevel(), 3);
    dispatch.dispatch(OscRouteGraph::AddNode((), (const_hnd, adjlist::NodeData::Effect(
        const_meta()
    ))).into()).unwrap();
    // Route constant output to modulo input (B)
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new(const_hnd, mod_hnd, EdgeWeight::new(pack_f32(2f32), 0, 1, 0)).unwrap(),)).into()).unwrap();
    
    // Read some data from ch=0.
    // This should be -3.5 % 2.0 = 0.5
    dispatch.dispatch(
        OscRenderer::RenderRange((), (0, 4, 1))
    .into()).unwrap();
    let rendered = rx.recv().unwrap();
    let exp = 0.5f32;
    assert_eq!(rendered, vec![exp, exp, exp, exp]);
}

#[test]
fn render_min() {
    let (mut dispatch, rx) = test_setup();

    // Create Modulo node (id=1)
    let min_hnd = NodeHandle::new_node(DagHandle::toplevel(), 1);
    dispatch.dispatch(OscRouteGraph::AddNode((), (min_hnd, adjlist::NodeData::Effect(
        min_meta()
    ))).into()).unwrap();
    // Connect delay output to master output.
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new_to_null(min_hnd, EdgeWeight::new(0, 0, 0, 0)),)).into()).unwrap();
    
    // Create Constant node (id=2)
    let const_hnd = NodeHandle::new_node(DagHandle::toplevel(), 2);
    dispatch.dispatch(OscRouteGraph::AddNode((), (const_hnd, adjlist::NodeData::Effect(
        const_meta()
    ))).into()).unwrap();
    // Route constant output to modulo input (A)
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new(const_hnd, min_hnd, EdgeWeight::new(pack_f32(-3.5f32), 0, 0, 0)).unwrap(),)).into()).unwrap();
    
    // Create Constant node (id=3)
    let const_hnd = NodeHandle::new_node(DagHandle::toplevel(), 3);
    dispatch.dispatch(OscRouteGraph::AddNode((), (const_hnd, adjlist::NodeData::Effect(
        const_meta()
    ))).into()).unwrap();
    // Route constant output to modulo input (B)
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new(const_hnd, min_hnd, EdgeWeight::new(pack_f32(2f32), 0, 1, 0)).unwrap(),)).into()).unwrap();
    
    // Read some data from ch=0.
    // This should be min(-3.5, 2.0) = -3.5
    dispatch.dispatch(
        OscRenderer::RenderRange((), (0, 4, 1))
    .into()).unwrap();
    let rendered = rx.recv().unwrap();
    let exp = -3.5f32;
    assert_eq!(rendered, vec![exp, exp, exp, exp]);
}
