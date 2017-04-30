//! Responsible for orchestrating all the different library components into
//! something cohesive. It effectively hides the rest of the library,
//! and all commands are meant to pass through this instead.

use std::collections::HashMap;
use osc_address::OscMessage;

use render::{Renderer, RefRenderer};
use routing::{DagHandle, Edge, NodeHandle, RouteGraph};
use routing::adjlist;

struct Dispatch {
    /// Contains the toplevel description of the audio being generated.
    routegraph: RouteGraph,
    /// Collection of all objects that are rendering the routegraph,
    /// mapped by id.
    renderers: HashMap<usize, Box<Renderer>>,
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
    AddNode((), (DagHandle, adjlist::NodeData)),
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


impl Dispatch {
    pub fn new() -> Dispatch {
        Dispatch {
            routegraph: RouteGraph::new(),
            renderers: HashMap::new(),
        }
    }
    /// Process the OSC message.
    fn dispatch(&mut self, msg: OscToplevel) {
        match msg {
            // TODO: move callbacks from inside routegraph to here.
            OscToplevel::RouteGraph((), rg_msg) => match rg_msg {
                OscRouteGraph::AddNode((), (handle, data)) => unimplemented!(),
                OscRouteGraph::AddEdge((), (edge,)) => self.routegraph.add_edge(edge).unwrap(),
                OscRouteGraph::DelNode((), (handle,)) => self.routegraph.del_node(handle).unwrap(),
                OscRouteGraph::DelEdge((), (edge,)) => self.routegraph.del_edge(edge),
            },
            OscToplevel::Renderer((), rend_msg) => match rend_msg {
                // TODO: create/delete the renderers
                OscRenderer::New((), (id,)) => { self.renderers.insert(id, Box::new(RefRenderer::new())); }
                OscRenderer::Del((), (id,)) => { self.renderers.remove(&id); }
            },
        }
    }
}
