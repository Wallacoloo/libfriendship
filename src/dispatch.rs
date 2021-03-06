//! Responsible for orchestrating all the different library components into
//! something cohesive. It effectively hides the rest of the library,
//! and all commands are meant to pass through this instead.

use std::path::Path;
use std::ops::Range;

use jagged_array::Jagged2;
use ndarray::{ArrayBase, Dim};

use client::Client;
use render::Renderer;
use resman::ResMan;
use routing::{Edge, Effect, NodeData, NodeHandle, RouteGraph, EffectId};
use routing::{effect, routegraph};

#[derive(Default, Debug)]
pub struct Dispatch<R, C> {
    /// Contains the toplevel description of the audio being generated.
    routegraph: RouteGraph,
    renderer: R,
    /// Resource manager. Knows where to find all data that might be stored
    /// outside the application.
    resman: ResMan,
    /// Where to send notifications of state changes,
    /// results from the renderer, etc.
    client: C,
}

/// OSC message to /<...>
#[derive(Debug, Clone)]
#[derive(OscMessage)]
pub enum OscToplevel {
    /// Send a message to the primary RouteGraph
    #[osc_address(address="routegraph")]
    RouteGraph((), OscRouteGraph),
    /// Send a message to one of the Renderers.
    #[osc_address(address="renderer")]
    Renderer((), OscRenderer),
    /// Send a message to the resource manager
    #[osc_address(address="resman")]
    ResMan((), OscResMan),
}

/// OSC message to /routegraph/<...>
#[derive(Debug, Clone)]
#[derive(OscMessage)]
pub enum OscRouteGraph {
    #[osc_address(address="add_node")]
    AddNode((), (NodeHandle, EffectId)),
    #[osc_address(address="add_edge")]
    AddEdge((), (Edge,)),
    #[osc_address(address="del_node")]
    DelNode((), (NodeHandle,)),
    #[osc_address(address="del_edge")]
    DelEdge((), (Edge,)),
    /// Query a node's metadata: its I/Os, etc.
    #[osc_address(address="query_meta")]
    QueryMeta((), (NodeHandle,)),
    /// Query a node's id: it's SHA, name, etc.
    #[osc_address(address="query_id")]
    QueryId((), (NodeHandle,)),
}

/// OSC message to /renderer/<...>
#[derive(Debug, Clone)]
#[derive(OscMessage)]
pub enum OscRenderer {
    /// Render a range of samples.
    /// First argument = which samples to render.
    /// Second arg = number of output slots to render.
    /// Third arg = inputs to be fed into slot0, 1, 2, ...,n.
    /// TODO: third argument should be made implicit based on RouteGraph metadata.
    /// TODO: second argument should be Jagged2; not Vec<Vec<f32>>
    #[osc_address(address="render")]
    RenderRange((), (Range<u64>, u32, Jagged2<f32>)),
}

/// OOSC message to /resman/<...>
#[derive(Debug, Clone)]
#[derive(OscMessage)]
pub enum OscResMan {
    /// Add another directory to watch when loading resources.
    #[osc_address(address="add_dir")]
    AddDir((), (String,)),
}


#[derive(Debug)]
pub enum Error {
    RouteGraphError(routegraph::Error),
    EffectError(effect::Error),
}

type ResultE<T> = Result<T, Error>;


impl<R, C> Dispatch<R, C> {
    pub fn new(renderer: R, client: C) -> Self {
        Self {
            routegraph: Default::default(),
            renderer,
            resman: Default::default(),
            client,
        }
    }
}

impl<R: Renderer, C: Client> Dispatch<R, C> {
    /// Process the OSC message.
    pub fn dispatch(&mut self, msg: OscToplevel) -> ResultE<()> {
        trace!("Dispatching message: {:?}", msg);
        match msg {
            OscToplevel::RouteGraph((), rg_msg) => match rg_msg {
                OscRouteGraph::AddNode((), (handle, id)) => {
                    let node_data = Effect::from_id(id, &self.resman)?;
                    self.routegraph.add_node(handle, node_data.clone())?;
                    self.on_add_node(&handle, &node_data);
                }
                OscRouteGraph::AddEdge((), (edge,)) => {
                    self.routegraph.add_edge(edge.clone())?;
                    self.on_add_edge(&edge);
                }
                OscRouteGraph::DelNode((), (handle,)) => {
                    self.routegraph.del_node(handle)?;
                    self.on_del_node(&handle);
                }
                OscRouteGraph::DelEdge((), (edge,)) => {
                    self.routegraph.del_edge(edge.clone());
                    self.on_del_edge(&edge);
                }
                OscRouteGraph::QueryMeta((), (handle,)) => {
                    if let Some(effect) = self.routegraph.get_data(&handle) {
                        self.client.node_meta(&handle, effect.meta());
                    } else {
                        warn!("QueryMeta: no such effect with handle: {:?}", handle);
                    }
                }
                OscRouteGraph::QueryId((), (handle,)) => {
                    if let Some(effect) = self.routegraph.get_data(&handle) {
                        self.client.node_id(&handle, &effect.id());
                    } else {
                        warn!("QueryId: no such effect with handle: {:?}", handle);
                    }
                }
            },
            OscToplevel::Renderer((), rend_msg) => match rend_msg {
                OscRenderer::RenderRange((), (range, num_slots, inputs)) => {
                    let mut buff = ArrayBase::zeros(Dim([num_slots as usize, (range.end-range.start) as usize]));
                    self.renderer.fill_buffer(&mut buff, range.start, inputs);
                    self.client.audio_rendered(buff, range.start);
                }
            },
            OscToplevel::ResMan((), res_msg) => match res_msg {
                OscResMan::AddDir((), (dir,)) => {
                    self.resman.add_dir(Path::new(&dir).to_path_buf());
                }
            }
        }
        Ok(())
    }
}

/// Conversion from `routegraph::Error` for use with the `?` operator
impl From<routegraph::Error> for Error {
    fn from(e: routegraph::Error) -> Self {
        Error::RouteGraphError(e)
    }
}

/// Conversion from `effect::Error` for use with the `?` operator
impl From<effect::Error> for Error {
    fn from(e: effect::Error) -> Self {
        Error::EffectError(e)
    }
}

/// Deterministic mapping from one OSC message to a container OSC message
impl From<OscRouteGraph> for OscToplevel {
    fn from(m: OscRouteGraph) -> Self {
        OscToplevel::RouteGraph((), m)
    }
}

/// Deterministic mapping from one OSC message to a container OSC message
impl From<OscRenderer> for OscToplevel {
    fn from(m: OscRenderer) -> Self {
        OscToplevel::Renderer((), m)
    }
}

/// Deterministic mapping from one OSC message to a container OSC message
impl From<OscResMan> for OscToplevel {
    fn from(m: OscResMan) -> Self {
        OscToplevel::ResMan((), m)
    }
}


/// Route callbacks to wherever they need to go
impl<R: Renderer, C> Dispatch<R, C> {
    fn on_add_node(&mut self, node: &NodeHandle, data: &NodeData) {
        self.renderer.on_add_node(node, data);
    }
    fn on_del_node(&mut self, node: &NodeHandle) {
        self.renderer.on_del_node(node);
    }
    fn on_add_edge(&mut self, edge: &Edge) {
        self.renderer.on_add_edge(edge);
    }
    fn on_del_edge(&mut self, edge: &Edge) {
        self.renderer.on_del_edge(edge);
    }
}
