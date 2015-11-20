use std::rc::Rc;

use super::node::YNode;
use super::send::Send;

pub trait Tree {
    /// Connect two nodes in the tree.
    /// If the send is ASrcSend / YSrcSend, this is the same as feeding
    /// external stimuli into the tree.
    fn add_send(&mut self, send: Send);

    /// set the nodes for which we are interested in the output PCM signals.
    /// Future calls to `step()` will return an array of samples corresponding
    /// to these nodes.
    fn watch_nodes(&mut self, outputs: &[Rc<YNode>]);
    /// Return the next buffer of samples related to the watched nodes.
    fn step(&mut self) -> &[f32];
}
