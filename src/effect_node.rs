use effects::effect::Effect;
use super::effect_send::EffectSend;

/// Describes a node on the effect tree, where leaves point up towards the root
/// Each leaf consists of an Effect and info on where to send that effect's
/// output.
pub struct EffectNode<'a> {
    effect : Box<Effect>,
    send : Option<EffectSend<'a>>,
}
/// Takes a node and encodes information that also allows traversal from the
/// root out to the leaves. In order to achieve this doubly-linked structure
/// safely, we need this class to be separate from EffectNode.
/// In actuality, EffectNode's are *always* physically contained within an
/// EffectNodeBi.
pub struct EffectNodeBi<'a> {
    node : EffectNode<'a>,
    children : Vec<Box<EffectNodeBi<'a>>>,
}

impl <'a> EffectNode<'a> {
    fn new(effect : Box<Effect>, send : Option<EffectSend<'a>>)
      -> EffectNode<'a> {
        EffectNode{ effect:effect, send:send }
    }
    /*pub fn effect(&'a self) -> &'a Effect {
        &*self.effect
    }*/
}
impl <'a> EffectNodeBi<'a> {
    pub fn new(effect : Box<Effect>, send : Option<EffectSend<'a>>)
      -> EffectNodeBi<'a> {
        EffectNodeBi::new_from_node(EffectNode::new(effect, send))
    }
    fn new_from_node(node : EffectNode) -> EffectNodeBi {
        EffectNodeBi{ node:node, children:vec![] }
    }
    /// Add a new child that sends its output to this node at the given slot.
    pub fn add_child(&'a mut self, child : Box<Effect>, slot : u32) {
        let send = EffectSend::new(&self.node, slot);
        let child_node = EffectNode::new(child, Some(send));
        let child_node_bi = EffectNodeBi::new_from_node(child_node);
        self.children.push(Box::new(child_node_bi));
    }
}