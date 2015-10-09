use std::rc::Rc;

use effects::effect::{EffectProcessIter, EffectRenderState};
use super::effect_node::EffectNode;
use super::partial::Partial;

/// Specifies an effect and a specific input slot to send Partials to.
#[derive(Clone)]
pub struct EffectSend<'a> {
    pub send_node : Rc<EffectNode<'a>>,
    send_slot : u32,
}

impl<'a> EffectSend<'a> {
    pub fn new(node : Rc<EffectNode<'a>>, slot : u32) -> EffectSend<'a> {
        EffectSend{ send_node:node, send_slot:slot }
    }
    pub fn send(&self, state : &mut EffectRenderState, partial : &Partial) -> EffectProcessIter {
        self.send_node.effect().process(state, partial, self.send_slot)
    }
    pub fn effect_node(&self) -> &Rc<EffectNode<'a>> {
        &self.send_node
    }
}