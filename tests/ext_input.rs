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
use libfriendship::render::RefRenderer;
use libfriendship::routing::{Edge, EdgeWeight, EffectId, NodeHandle};
use libfriendship::util::pack_f32;


struct MyClient {
    /// Where to send the rendered audio.
    tx: Sender<Array2<f32>>,
}
impl Client for MyClient {
    fn audio_rendered(&mut self, buffer: Array2<f32>, _idx: u64) {
        self.tx.send(buffer).unwrap();
    }
}

fn test_setup() -> (Dispatch<RefRenderer, MyClient>, Receiver<Array2<f32>>) {
    let (tx, rx) = channel();
    let dispatch = Dispatch::new(RefRenderer::default(), MyClient{ tx });
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
    // empty inputs are implicitly 0.
    assert_eq!(rendered, array![[0f32, 1f32, 2f32, 0f32]]);

    // Seek to zero and render more
    dispatch.dispatch(
        OscRenderer::RenderRange((), (0..4, 1, Default::default()))
    .into()).unwrap();
    // Seeking implicitly zeros the inputs
    let rendered = rx.recv().unwrap();
    assert_eq!(rendered, array![[0f32, 0f32, 0f32, 0f32]]);
}

