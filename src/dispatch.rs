//! Responsible for orchestrating all the different library components into
//! something cohesive. It effectively hides the rest of the library,
//! and all commands are meant to pass through this instead.

use std::collections::HashMap;
use osc_address::OscMessage;

use render::{Renderer, RefRenderer};
use resman::ResMan;
use routing;
use routing::{Edge, Effect, NodeHandle, RouteGraph};
use routing::{adjlist, effect, routegraph};

struct Dispatch {
    /// Contains the toplevel description of the audio being generated.
    routegraph: RouteGraph,
    /// Collection of all objects that are rendering the routegraph,
    /// mapped by id.
    renderers: HashMap<usize, Box<Renderer>>,
    /// Resource manager. Knows where to find all data that might be stored
    /// outside the application.
    resman: ResMan,
}

#[derive(OscMessage)]
enum OscToplevel {
    /// Send a message to the primary RouteGraph
    #[osc_address(address="routegraph")]
    RouteGraph((), OscRouteGraph),
    /// Send a message to one of the Renderers.
    #[osc_address(address="renderer")]
    Renderer((), OscRenderer),
}

#[derive(OscMessage)]
enum OscRouteGraph {
    #[osc_address(address="add_node")]
    AddNode((), (NodeHandle, adjlist::NodeData)),
    #[osc_address(address="add_edge")]
    AddEdge((), (Edge,)),
    #[osc_address(address="del_node")]
    DelNode((), (NodeHandle,)),
    #[osc_address(address="del_edge")]
    DelEdge((), (Edge,)),
}

#[derive(OscMessage)]
enum OscRenderer {
    /// Create a new renderer with id=msg payload.
    /// Currently instantiates a reference renderer with default settings.
    /// TODO: this should someday accept the typename of a renderer.
    #[osc_address(address="new")]
    New((), (usize,)),
    /// Delete the renderer that has id=msg payload.
    #[osc_address(address="del")]
    Del((), (usize,)),
}

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
        }
    }
    /// Process the OSC message.
    fn dispatch(&mut self, msg: OscToplevel) -> ResultE<()> {
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
                OscRenderer::New((), (id,)) => { self.renderers.insert(id, Box::new(RefRenderer::new())); }
                OscRenderer::Del((), (id,)) => { self.renderers.remove(&id); }
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
