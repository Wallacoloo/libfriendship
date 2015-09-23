use effects::effect::EffectRenderState;
use super::effect_node::EffectNode;
use super::partial::Partial;

/// Specifies an effect and a specific input slot to send Partials to.
#[derive(Copy, Clone)]
pub struct EffectSend<'a> {
    pub send_node : &'a EffectNode<'a>,
    send_slot : u32,
}

impl<'a> EffectSend<'a> {
    pub fn new(node : &'a EffectNode, slot : u32) -> EffectSend<'a> {
        EffectSend{ send_node:node, send_slot:slot }
    }
    pub fn send(&self, state : &mut EffectRenderState, partial : &Partial) -> Box<Iterator<Item=Partial>> {
        self.send_node.effect().process(state, partial, self.send_slot)
    }
    pub fn effect_node(&self) -> &EffectNode<'a> {
        self.send_node
    }
}