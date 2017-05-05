//! Test rendering of primitive effects, through the Dispatch interface.

extern crate libfriendship;
extern crate url;

use std::sync::mpsc::{channel, Receiver, Sender};

use url::Url;

use libfriendship::{Dispatch, Client};
use libfriendship::dispatch::{OscRouteGraph, OscRenderer};
use libfriendship::render::RefRenderer;
use libfriendship::routing::{adjlist, DagHandle, Edge, EdgeWeight, EffectMeta, NodeHandle};


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
        EffectMeta::new("Constant".to_string(), None, [Url::parse("primitive:///Constant?value=0.5").unwrap()].iter().cloned())
    ))).into()).unwrap();
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new_to_null(handle, EdgeWeight::new(0, 0, 0, 0)),)).into()).unwrap();
    
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
        EffectMeta::new("Delay".to_string(), None, [Url::parse("primitive:///Delay?frames=2").unwrap()].iter().cloned())
    ))).into()).unwrap();
    // Connect delay output to master output.
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new_to_null(delay_hnd, EdgeWeight::new(0, 0, 0, 0)),)).into()).unwrap();

    // Create Constant node (id=2)
    let const_hnd = NodeHandle::new_node(DagHandle::toplevel(), 2);
    dispatch.dispatch(OscRouteGraph::AddNode((), (const_hnd, adjlist::NodeData::Effect(
        EffectMeta::new("Constant".to_string(), None, [Url::parse("primitive:///Constant?value=0.5").unwrap()].iter().cloned())
    ))).into()).unwrap();
    // Route constant output to delay input
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new(const_hnd, delay_hnd, EdgeWeight::new(0, 0, 0, 0)).unwrap(),)).into()).unwrap();
    
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
        EffectMeta::new("Multiply".to_string(), None, [Url::parse("primitive:///Multiply").unwrap()].iter().cloned())
    ))).into()).unwrap();
    // Connect delay output to master output.
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new_to_null(mult_hnd, EdgeWeight::new(0, 0, 0, 0)),)).into()).unwrap();
    
    // Create Constant node (id=2)
    let const_hnd = NodeHandle::new_node(DagHandle::toplevel(), 2);
    dispatch.dispatch(OscRouteGraph::AddNode((), (const_hnd, adjlist::NodeData::Effect(
        EffectMeta::new("Constant".to_string(), None, [Url::parse("primitive:///Constant?value=0.5").unwrap()].iter().cloned())
    ))).into()).unwrap();
    // Route constant output to multiply input (A)
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new(const_hnd, mult_hnd, EdgeWeight::new(0, 0, 0, 0)).unwrap(),)).into()).unwrap();
    
    // Create Constant node (id=3)
    let const_hnd = NodeHandle::new_node(DagHandle::toplevel(), 3);
    dispatch.dispatch(OscRouteGraph::AddNode((), (const_hnd, adjlist::NodeData::Effect(
        EffectMeta::new("Constant".to_string(), None, [Url::parse("primitive:///Constant?value=-3.0").unwrap()].iter().cloned())
    ))).into()).unwrap();
    // Route constant output to multiply input (B)
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new(const_hnd, mult_hnd, EdgeWeight::new(0, 0, 1, 0)).unwrap(),)).into()).unwrap();
    
    // Read some data from ch=0.
    // This should be 0.5 * -3.0 = -1.5
    dispatch.dispatch(
        OscRenderer::RenderRange((), (0, 4, 1))
    .into()).unwrap();
    let rendered = rx.recv().unwrap();
    assert_eq!(rendered, vec![-1.5f32, -1.5f32, -1.5f32, -1.5f32]);
}
