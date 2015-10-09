use std::rc::Rc;

use super::effect_node::EffectNode;

/// Specifies an effect and a specific input slot to send Partials to.
#[derive(Clone)]
pub struct EffectSend<'a> {
    send_node : Rc<EffectNode<'a>>,
    send_slot : u32,
}

impl<'a> EffectSend<'a> {
    pub fn new(node : Rc<EffectNode<'a>>, slot : u32) -> EffectSend<'a> {
        EffectSend{ send_node:node, send_slot:slot }
    }
    pub fn effect_node(&self) -> &Rc<EffectNode<'a>> {
        &self.send_node
    }
    pub fn send_slot(&self) -> u32 {
        self.send_slot
    }
}