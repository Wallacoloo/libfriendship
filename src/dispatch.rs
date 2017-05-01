//! Responsible for orchestrating all the different library components into
//! something cohesive. It effectively hides the rest of the library,
//! and all commands are meant to pass through this instead.

use std::collections::HashMap;
use osc_address::OscMessage;

use client::Client;
use render::{Renderer, RefRenderer};
use resman::ResMan;
use routing;
use routing::{Edge, Effect, NodeHandle, RouteGraph};
use routing::{adjlist, effect, routegraph};

pub struct Dispatch {
    /// Contains the toplevel description of the audio being generated.
    routegraph: RouteGraph,
    /// Collection of all objects that are rendering the routegraph,
    /// mapped by id.
    renderers: HashMap<u32, Box<Renderer>>,
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
    /// Create a new renderer with id=msg payload.
    /// Currently instantiates a reference renderer with default settings.
    /// TODO: this should someday accept the typename of a renderer.
    #[osc_address(address="new")]
    New((), (u32,)),
    /// Delete the renderer that has id=msg payload.
    #[osc_address(address="del")]
    Del((), (u32,)),
    ById(u32, OscRendererById),
}

/// OSC message to /renderer/<renderer_id>/<...>
#[derive(OscMessage)]
pub enum OscRendererById {
    /// Render a range of samples from [a, b)
    /// Last argument indicates the number of channels to render.
    /// TODO: The channel count should become a property of the RouteGraph.
    #[osc_address(address="render")]
    RenderRange((), (u64, u64, u8)),
}

#[derive(Debug)]
pub enum Error {
    RouteGraphError(routegraph::Error),
    EffectError(effect::Error),
}

type ResultE<T> = Result<T, Error>;


impl Dispatch {
    pub fn new() -> Dispatch {
        Dispatch {
            routegraph: RouteGraph::new(),
            renderers: HashMap::new(),
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
                    for watcher in self.renderers.values_mut() {
                        watcher.on_add_node(&handle, &node_data);
                    }
                }
                OscRouteGraph::AddEdge((), (edge,)) => {
                    self.routegraph.add_edge(edge.clone())?;
                    for watcher in self.renderers.values_mut() {
                        watcher.on_add_edge(&edge);
                    }
                }
                OscRouteGraph::DelNode((), (handle,)) => {
                    self.routegraph.del_node(handle.clone())?;
                    for watcher in self.renderers.values_mut() {
                        watcher.on_del_node(&handle);
                    }
                }
                OscRouteGraph::DelEdge((), (edge,)) => {
                    self.routegraph.del_edge(edge.clone());
                    for watcher in self.renderers.values_mut() {
                        watcher.on_del_edge(&edge);
                    }
                }
            },
            OscToplevel::Renderer((), rend_msg) => match rend_msg {
                // TODO: upon creation, we need to read the current graph into the new renderer.
                OscRenderer::New((), (id,)) => {
                    self.renderers.insert(id, Box::new(RefRenderer::new()));
                }
                OscRenderer::Del((), (id,)) => {
                    self.renderers.remove(&id);
                }
                OscRenderer::ById(id, rend_msg) => match rend_msg {
                    OscRendererById::RenderRange((), (start, stop, num_ch)) => {
                        // Avoid underflows if the range isn't positive.
                        if stop < start { return Ok(()); }
                        let size = (stop-start)*(num_ch as u64);
                        let mut buff: Vec<f32> = (0..size).map(|i| { 0f32 }).collect();
                        // TODO: handle index error
                        self.renderers.get_mut(&id).unwrap().fill_buffer(&mut buff, start, num_ch);
                        self.audio_rendered(id, &buff, start, num_ch);
                    }
                }
            },
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


/// Calling any Client method on Dispatch routes it to all the Dispatch's own
/// clients. This is meant to be used internally.
impl Dispatch {
    fn audio_rendered(&mut self, renderer_id: u32, buffer: &[f32], idx: u64, num_ch: u8) {
        for c in self.clients.values_mut() {
            c.audio_rendered(renderer_id, buffer, idx, num_ch);
        }
    }
}
