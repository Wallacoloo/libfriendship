//! Test rendering of primitive effects, through the Dispatch interface.

extern crate libfriendship;
extern crate url;

use std::sync::mpsc::{channel, Sender};

use libfriendship::{Dispatch, Client};
use libfriendship::dispatch::{OscToplevel, OscRouteGraph, OscRenderer, OscRendererById};
use libfriendship::routing::{adjlist, DagHandle, Edge, EdgeWeight, EffectMeta, NodeHandle};
use url::Url;


struct MyClient {
    /// Where to send the rendered audio.
    tx: Sender<Vec<f32>>,
}
impl Client for MyClient {
    fn audio_rendered(&mut self, renderer_id: u32, buffer: &[f32], idx: u64, num_ch: u8) {
        self.tx.send(buffer.iter().map(|x| *x).collect());
    }
}

#[test]
fn render_zeros() {
    let (tx, rx) = channel();
    let client = Box::new(MyClient{ tx });
    let mut dispatch = Dispatch::new();
    dispatch.register_client(client);
    // Create the renderer
    let rend_id = 1;
    dispatch.dispatch(OscRenderer::New((), (rend_id,)).into()).unwrap();

    // Read some data from ch=0.
    // This should be all zeros because we have no data being rendered.
    dispatch.dispatch(OscRenderer::ById(rend_id,
        OscRendererById::RenderRange((), (0, 4, 1))
    ).into()).unwrap();
    let rendered = rx.recv().unwrap();
    assert_eq!(rendered, vec![0f32, 0f32, 0f32, 0f32]);

    // Route a constant into ch=0.
    let handle = NodeHandle::new_node(DagHandle::toplevel(), 1);
    dispatch.dispatch(OscRouteGraph::AddNode((), (handle, adjlist::NodeData::Effect(
        EffectMeta::new("Constant".to_string(), None, [Url::parse("primitive:///Constant?value=0.5").unwrap()].iter().cloned())
    ))).into()).unwrap();
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new_to_null(handle, EdgeWeight::new(0, 0, 0, 0)),)).into()).unwrap();
    
    // Read some data from ch=0.
    // This should be all 0.5 because of the new node we added.
    dispatch.dispatch(OscRenderer::ById(rend_id,
        OscRendererById::RenderRange((), (0, 4, 1))
    ).into()).unwrap();
    let rendered = rx.recv().unwrap();
    assert_eq!(rendered, vec![0.5f32, 0.5f32, 0.5f32, 0.5f32]);
}

