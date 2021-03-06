//! Test loading of effect definitions from the disk.

#[macro_use] extern crate libfriendship;
extern crate digest;
#[macro_use] extern crate ndarray;
extern crate serde_json;
extern crate sha2;
extern crate tempdir;
extern crate url;

use std::fs::File;
use std::sync::mpsc::{channel, Receiver, Sender};

use digest::Digest;
use ndarray::Array2;
use sha2::Sha256;
use tempdir::TempDir;
use url::Url;

use libfriendship::{Dispatch, Client};
use libfriendship::dispatch::{OscRouteGraph, OscRenderer, OscResMan};
use libfriendship::render::SparkleRenderer;
use libfriendship::routing::{NodeHandle, Edge, EdgeWeight, EffectId, EffectDesc, EffectMeta, EffectInput, EffectOutput};
use libfriendship::routing::AdjList;

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

fn create_multby2() -> EffectDesc {
    let mult_hnd = NodeHandle::new(1);
    let mult_data = EffectId::new("Multiply".into(), None, vec![Url::parse("primitive:///Multiply").unwrap()]);
    let const_hnd = NodeHandle::new(2);
    let const_data = EffectId::new("Constant".into(), None, vec![Url::parse("primitive:///F32Constant").unwrap()]);

    let nodes = vec![(mult_hnd, mult_data), (const_hnd, const_data)];

    // input data sent to multiply (A)
    let edge_in = Edge::new_from_null(mult_hnd, EdgeWeight::new(0, 0));
    // multiply out sent to effect out
    let edge_out = Edge::new_to_null(mult_hnd, EdgeWeight::new(0, 0));
    // const data sent to multiply (B)
    let edge_const = Edge::new(const_hnd, mult_hnd, EdgeWeight::new((5.0f32).to_bits(), 1));

    let edges = vec![edge_in, edge_out, edge_const];

    let list = AdjList{ nodes, edges };
    let meta = EffectMeta::new("MulBy2".into(), None,
        vec![ EffectInput::new("source".into(), 0) ],
        vec![ EffectOutput::new("result".into(), 0) ],
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
        OscResMan::AddDir((), (dir.path().to_str().unwrap().into(),)).into()
    ).unwrap();

    // Write the effect definition to file
    let mulby2_path = dir.path().join("mulby2.fnd");
    let mulby2_file = File::create(mulby2_path.clone()).unwrap();
    serde_json::to_writer(mulby2_file, &mulby2_desc).unwrap();

    // Determine the hash of our file
    let mut mulby2_file = File::open(mulby2_path).unwrap();
    let hash_result = Sha256::digest_reader(&mut mulby2_file).unwrap();
    let mut sha: [u8; 32] = Default::default();
    sha.copy_from_slice(hash_result.as_slice());

    // Create the MulBy2 node (id=1)
    let mul_hnd = NodeHandle::new(1);
    dispatch.dispatch(OscRouteGraph::AddNode((), (mul_hnd,
        EffectId::new("MulBy2".into(), Some(sha), None)
    )).into()).unwrap();
    // Connect MulBy2 output to master output.
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new_to_null(mul_hnd, EdgeWeight::new(0, 0)),)).into()).unwrap();
    
    // Create Constant node (id=2)
    let const_hnd = NodeHandle::new(2);
    dispatch.dispatch(OscRouteGraph::AddNode((), (const_hnd,
        EffectId::new("Constant".into(), None, vec![Url::parse("primitive:///F32Constant").unwrap()])
    )).into()).unwrap();
    // Route constant output to mul input
    dispatch.dispatch(OscRouteGraph::AddEdge((), (Edge::new(const_hnd, mul_hnd, EdgeWeight::new((0.5f32).to_bits(), 0)),)).into()).unwrap();
    
    // Read some data from ch=0.
    // This should be 0.5*5 = [2.5, 2.5, 2.5, 2.5]
    dispatch.dispatch(
        OscRenderer::RenderRange((), (0..4, 1, Default::default()))
    .into()).unwrap();
    let rendered = rx.recv().unwrap();
    assert_eq!(rendered, array![[2.5f32, 2.5f32, 2.5f32, 2.5f32]]);
}

