/// Type that gets notifications whenever a component of a `RouteGraph` changes

use super::routegraph::{Edge, NodeData, NodeHandle};
pub trait GraphWatcher {
    fn on_add_node(&mut self, node: &NodeHandle, data: &NodeData);
    fn on_del_node(&mut self, node: &NodeHandle);
    fn on_add_edge(&mut self, edge: &Edge);
    fn on_del_edge(&mut self, edge: &Edge);
}

