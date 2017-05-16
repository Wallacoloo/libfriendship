//! Test loading of effect definitions from the disk.

extern crate libfriendship;
extern crate digest;
extern crate serde_json;
extern crate sha2;
extern crate tempdir;
extern crate url;

use std::fs::File;
use std::sync::mpsc::{channel, Receiver, Sender};

use digest::digest_reader;
use sha2::Sha256;
use tempdir::TempDir;
use url::Url;

use libfriendship::{Dispatch, Client};
use libfriendship::dispatch::{OscRouteGraph, OscRenderer, OscResMan};
use libfriendship::render::RefRenderer;
use libfriendship::routing::{adjlist, NodeHandle, DagHandle, Edge, EdgeWeight, EffectId, EffectDesc, EffectMeta, EffectInput, EffectOutput};
use libfriendship::routing::AdjList;
use libfriendship::util::pack_f32;

struct MyClient {
    /// Where to send the rendered audio.
    tx: Sender<Vec<f32>>,
}
impl Client for MyClient {
    fn audio_rendered(&mut self, buffer: &[f32], _idx: u64, _num_slots: u32) {
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

fn create_multby2() -> EffectDesc {
    let mult_hnd = NodeHandle::new_node(DagHandle::toplevel(), 1);
    let mult_data = adjlist::NodeData::Effect(
        EffectId::new("Multiply".into(), None, vec![Url::parse("primitive:///Multiply").unwrap()])
    );
    let const_hnd = NodeHandle::new_node(DagHandle::toplevel(), 2);
    let const_data = adjlist::NodeData::Effect(
        EffectId::new("Constant".into(), None, vec![Url::parse("primitive:///F32Constant").unwrap()])
    );

    let nodes = [(mult_hnd, mult_data), (const_hnd, const_data)];

    // input data sent to multiply (A)
    let edge_in = Edge::new_from_null(mult_hnd, EdgeWeight::new(0, 0));
    // multiply out sent to effect out
    let edge_out = Edge::new_to_null(mult_hnd, EdgeWeight::new(0, 0));
    // const data sent to multiply (B)
    let edge_const = Edge::new(const_hnd, mult_hnd, EdgeWeight::new(pack_f32(5.0f32), 1)).unwrap();

    let edges = [edge_in, edge_out, edge_const];

    let list = AdjList {
        nodes: nodes.iter().cloned().collect(),
        edges: edges.iter().cloned().collect(),
    };
    let meta = EffectMeta::new("MulBy2".into(), None,
        [(0, EffectInput::new("source".into(), 0))].iter().cloned().collect(),
        [(0, EffectOutput::new("result".into(), 0))].iter().cloned().collect(),
    );
    EffectDesc::new(meta, list)
}

#[test]
fn load_multby2() {
    let (mut dispatch, rx) = test_setup();
    let dir = TempDir::new("libfriendship").unwrap();
    let mulby2_desc = create_multby2();

    // Add the temp dir as a search dir
    dispatch.dispatch(
        OscResMan::AddDir((), (dir.path().to_str().unwrap().to_string(),)).into()
    ).unwrap();

    // Write the effect definition to file
    let mulby2_path = dir.path().join("mulby2.fnd");
    let mulby2_file = File::create(mulby2_path.clone()).unwrap();
    serde_json::to_writer(mulby2_file, &mulby2_desc).unwrap();

    // Determine the hash of our file
    let mut mulby2_file = File::open(mulby2_path).unwrap();
    let hash_result = digest_reader::<Sha256>(&mut mulby2_file).unwrap();
    let mut sha: [u8; 32] = Default::default();
    sha.copy_from_slice(hash_result.as_slice());

    // Create the MulBy2 node (id=1)
    let mul_hnd = NodeHandle::new_node(DagHandle::toplevel(), 1);
    dispatch.dispatch(OscRouteGraph::AddNode((), (mul_hnd, adjlist::NodeData::Effect(
        EffectId::new("MulBy2".into(), Some(sha), None)
    ))).into()).unwrap();
    // Connect MulBy2 output to master output.
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new_to_null(mul_hnd, EdgeWeight::new(0, 0)),)).into()).unwrap();
    
    // Create Constant node (id=2)
    let const_hnd = NodeHandle::new_node(DagHandle::toplevel(), 2);
    dispatch.dispatch(OscRouteGraph::AddNode((), (const_hnd, adjlist::NodeData::Effect(
        EffectId::new("Constant".into(), None, vec![Url::parse("primitive:///F32Constant").unwrap()])
    ))).into()).unwrap();
    // Route constant output to mul input
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new(const_hnd, mul_hnd, EdgeWeight::new(pack_f32(0.5f32), 0)).unwrap(),)).into()).unwrap();
    
    // Read some data from ch=0.
    // This should be 0.5*5 = [2.5, 2.5, 2.5, 2.5]
    dispatch.dispatch(
        OscRenderer::RenderRange((), (0, 4, 1))
    .into()).unwrap();
    let rendered = rx.recv().unwrap();
    assert_eq!(rendered, vec![2.5f32, 2.5f32, 2.5f32, 2.5f32]);
}

