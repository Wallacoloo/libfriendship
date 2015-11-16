use std::rc::Rc;

use automation::Automation;
use partial::Partial;

use super::node::{ANode, YNode, ASource, YSource, YSink};
use super::send::Send;

trait Tree {
    /// Only specific nodes may spontaneously generate signals.
    /// These must be declared using add_input_a/add_input_y.
    /// The return value can be used with feed_a or feed_b.
    fn add_input_a(&mut self, input: Rc<ANode>) -> Rc<ASource>;
    fn add_input_y(&mut self, input: Rc<YNode>) -> Rc<YSource>;
    /// Only specific nodes may sink audio. These must be explicitly declared.
    /// They can then be used with sink_y
    fn add_sink_y(&mut self, out: Rc<YNode>) -> Rc<YSink>;
    /// Connect two nodes in the tree.
    fn add_send(&mut self, send: Send);

    /// add Automation external stimuli to the tree.
    fn feed_a(&mut self, a: Automation, src: Rc<ASource>);
    /// add Partial external stimuli to the tree.
    fn feed_y(&mut self, y: Partial, src: Rc<YSource>);
    /// Poll the next Partial to arrive at the provided Sink.
    /// Will return None if there are no new partials to be read.
    fn sink_y(&mut self, sink: Rc<YSink>) -> Option<Partial>;
}
