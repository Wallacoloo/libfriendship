use super::effect_node::EffectNode;

/// Specifies an effect and a specific input slot to send Partials to.
#[derive(Copy, Clone)]
pub struct EffectSend<'a> {
    send_node : &'a EffectNode<'a>,
    send_slot : u32,
}