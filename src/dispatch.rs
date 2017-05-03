//! Responsible for orchestrating all the different library components into
//! something cohesive. It effectively hides the rest of the library,
//! and all commands are meant to pass through this instead.

use std::collections::HashMap;
use std::path::Path;

use client::Client;
use render::Renderer;
use resman::ResMan;
use routing;
use routing::{Edge, Effect, NodeData, NodeHandle, RouteGraph};
use routing::{adjlist, effect, routegraph};

pub struct Dispatch<R> {
    /// Contains the toplevel description of the audio being generated.
    routegraph: RouteGraph,
    renderer: R,
    /// Resource manager. Knows where to find all data that might be stored
    /// outside the application.
    resman: ResMan,
    /// All clients that wish to receive notifications of state change or
    /// reults from the renderer, etc.
    clients: HashMap<u32, Box<Client>>,
}

/// OSC message to /<...>
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
#[derive(OscMessage)]
pub enum OscRouteGraph {
    #[osc_address(address="add_node")]
    AddNode((), (NodeHandle, adjlist::NodeData)),
    #[osc_address(address="add_edge")]
    AddEdge((), (Edge,)),
    #[osc_address(address="del_node")]
    DelNode((), (NodeHandle,)),
    #[osc_address(address="del_edge")]
    DelEdge((), (Edge,)),
}

/// OSC message to /renderer/<...>
#[derive(OscMessage)]
pub enum OscRenderer {
    /// Render a range of samples from [a, b)
    /// Last argument indicates the number of channels to render.
    /// TODO: The channel count should become a property of the RouteGraph.
    #[osc_address(address="render")]
    RenderRange((), (u64, u64, u8)),
}

/// OOSC message to /resman/<...>
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


impl<R: Renderer + Default> Dispatch<R> {
    pub fn new() -> Dispatch<R> {
        Dispatch {
            routegraph: RouteGraph::new(),
            renderer: Default::default(),
            resman: ResMan::new(),
            clients: HashMap::new(),
        }
    }
    /// Registers the client to receive event messages.
    /// Returns the id that has been assigned to the client.
    pub fn register_client(&mut self, c: Box<Client>) -> u32 {
        let id = 1+self.clients.keys().max().unwrap_or(&0);
        self.clients.insert(id, c);
        id
    }
    /// Process the OSC message.
    pub fn dispatch(&mut self, msg: OscToplevel) -> ResultE<()> {
        match msg {
            OscToplevel::RouteGraph((), rg_msg) => match rg_msg {
                OscRouteGraph::AddNode((), (handle, data)) => {
                    let node_data = match data {
                        adjlist::NodeData::Effect(meta) =>
                            routing::NodeData::Effect(Effect::from_meta(meta, &self.resman)?),
                        adjlist::NodeData::Graph(dag_handle) =>
                            routing::NodeData::Graph(dag_handle),
                    };
                    self.routegraph.add_node(handle.clone(), node_data.clone())?;
                    self.on_add_node(&handle, &node_data);
                }
                OscRouteGraph::AddEdge((), (edge,)) => {
                    self.routegraph.add_edge(edge.clone())?;
                    self.on_add_edge(&edge);
                }
                OscRouteGraph::DelNode((), (handle,)) => {
                    self.routegraph.del_node(handle.clone())?;
                    self.on_del_node(&handle);
                }
                OscRouteGraph::DelEdge((), (edge,)) => {
                    self.routegraph.del_edge(edge.clone());
                    self.on_del_edge(&edge);
                }
            },
            OscToplevel::Renderer((), rend_msg) => match rend_msg {
                OscRenderer::RenderRange((), (start, stop, num_ch)) => {
                    // Avoid underflows if the range isn't positive.
                    if stop < start { return Ok(()); }
                    let size = (stop-start)*(num_ch as u64);
                    let mut buff: Vec<f32> = (0..size).map(|_| { 0f32 }).collect();
                    self.renderer.fill_buffer(&mut buff, start, num_ch);
                    self.audio_rendered(&buff, start, num_ch);
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


/// Calling any Client method on Dispatch routes it to all the Dispatch's own
/// clients.
impl<R: Renderer + Default> Dispatch<R> {
    fn audio_rendered(&mut self, buffer: &[f32], idx: u64, num_ch: u8) {
        for c in self.clients.values_mut() {
            c.audio_rendered(buffer, idx, num_ch);
        }
    }
}

/// Calling any GraphWatcher method on Dispatch routes it to all the
/// Dispatch's own GraphWatchers.
impl<R: Renderer + Default> Dispatch<R> {
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
