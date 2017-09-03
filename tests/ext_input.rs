//! Test rendering of primitive effects, through the Dispatch interface.

extern crate jagged_array;
extern crate libfriendship;
#[macro_use] extern crate ndarray;
extern crate url;

use std::sync::mpsc::{channel, Receiver, Sender};

use jagged_array::Jagged2Builder;
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

/// Return the `EffectId` that universally represents `F32Constant` nodes.
fn const_id() -> EffectId {
    EffectId::new("F32Constant".into(), None, vec![Url::parse("primitive:///F32Constant").unwrap()])
}

#[test]
fn render_passthrough() {
    let (mut dispatch, rx) = test_setup();

    // Add an edge from input to output
    dispatch.dispatch(OscRouteGraph::AddEdge((),
        (Edge::new_to_null(NodeHandle::toplevel(), EdgeWeight::new(0, 0)),)
    ).into()).unwrap();

    // Read some data from ch=0.
    let mut builder = Jagged2Builder::new();
    builder.extend(&[1f32, 2f32, 3f32, 4f32]);
    dispatch.dispatch(
        OscRenderer::RenderRange((), (0..4, 1, builder.into()))
    .into()).unwrap();
    let rendered = rx.recv().unwrap();
    assert_eq!(rendered, array![[1f32, 2f32, 3f32, 4f32]]);

    // Read some more data
    let mut builder = Jagged2Builder::new();
    builder.extend(&[0f32, 1f32, 2f32]);
    dispatch.dispatch(
        OscRenderer::RenderRange((), (4..8, 1, builder.into()))
    .into()).unwrap();
    let rendered = rx.recv().unwrap();
    // empty inputs take on their last known value.
    assert_eq!(rendered, array![[0f32, 1f32, 2f32, 2f32]]);

    // Seek to zero and render more
    dispatch.dispatch(
        OscRenderer::RenderRange((), (0..4, 1, Default::default()))
    .into()).unwrap();
    // Seeking implicitly zeros the inputs
    let rendered = rx.recv().unwrap();
    assert_eq!(rendered, array![[0f32, 0f32, 0f32, 0f32]]);
}

#[test]
fn render_delay() {
    let (mut dispatch, rx) = test_setup();

    // Create delay node (id=1)
    let delay_hnd = NodeHandle::new(1);
    dispatch.dispatch(OscRouteGraph::AddNode( (), (delay_hnd, delay_id()) ).into()).unwrap();
    // Connect delay output to master output.
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new_to_null(delay_hnd, EdgeWeight::new(0, 0)),)).into()).unwrap();

    // Add an edge from input to output
    dispatch.dispatch(OscRouteGraph::AddEdge((),
        (Edge::new_from_null(delay_hnd, EdgeWeight::new(0, 0)),)
    ).into()).unwrap();

    // Read some data from ch=0.
    let mut builder = Jagged2Builder::new();
    builder.extend(&[1f32, 2f32, 3f32, 4f32]);
    dispatch.dispatch(
        OscRenderer::RenderRange((), (0..4, 1, builder.into()))
    .into()).unwrap();
    let rendered = rx.recv().unwrap();
    assert_eq!(rendered, array![[1f32, 2f32, 3f32, 4f32]]);


    // Now delay by 1 frame.
    // Create Constant node (id=2)
    let const_hnd = NodeHandle::new(2);
    dispatch.dispatch(OscRouteGraph::AddNode( (), (const_hnd, const_id()) ).into()).unwrap();
    // Route constant output to delay input
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new(const_hnd, delay_hnd, EdgeWeight::new((1f32).to_bits(), 1)),)).into()).unwrap();
    // Read some data from ch=0.
    let mut builder = Jagged2Builder::new();
    builder.extend(&[1f32, 2f32, 3f32, 4f32]);
    dispatch.dispatch(
        OscRenderer::RenderRange((), (4..8, 1, builder.into()))
    .into()).unwrap();
    let rendered = rx.recv().unwrap();
    assert_eq!(rendered, array![[4f32, 1f32, 2f32, 3f32]]);
}

