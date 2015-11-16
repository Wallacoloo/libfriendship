use std::rc::Rc;

use automation::Automation;
use partial::Partial;

use super::node::{ANode, YNode, ASrcNode, YSrcNode, YSinkNode};
use super::send::Send;

trait Tree {
    /// Connect two nodes in the tree.
    fn add_send(&mut self, send: Send);

    /// add Automation external stimuli to the tree.
    fn feed_a(&mut self, a: Automation, src: Rc<ASrcNode>);
    /// add Partial external stimuli to the tree.
    fn feed_y(&mut self, y: Partial, src: Rc<YSrcNode>);
    /// Poll the next Partial to arrive at the provided Sink.
    /// Will return None if there are no new partials to be read.
    fn sink_y(&mut self, sink: Rc<YSinkNode>) -> Option<Partial>;
}
