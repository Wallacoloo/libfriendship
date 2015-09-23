use std::cmp::Ordering;
use std::hash::{self, Hash};
use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT};
use std::sync::atomic;

use effects::effect::Effect;
use super::effect_send::EffectSend;

/// This variable is incremented every time a new EffectNode is created in
/// order to give it a unique tag.
static mut effect_id_gen : AtomicUsize = ATOMIC_USIZE_INIT;

/// Describes a node on the effect tree, where leaves point up towards the root
/// Each leaf consists of an Effect and info on where to send that effect's
/// output.
pub struct EffectNode<'a> {
    /// in order to store this structure as a key in an associative container,
    /// e.g. a HashMap, we need some way to uniquely tag effects. Because
    /// size_of::<Effect>() == 0, we cannot depend upon the hash of the pointer
    id : u32,
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

impl<'a> EffectNode<'a> {
    fn new(effect : Box<Effect>, send : Option<EffectSend<'a>>)
      -> EffectNode<'a> {
        let id = unsafe { effect_id_gen.fetch_add(1, atomic::Ordering::SeqCst) };
        EffectNode{ effect:effect, send:send, id:id as u32 }
    }
    pub fn effect(&self) -> &Effect {
        &*self.effect
    }
    pub fn effect_send(&self) -> &Option<EffectSend<'a>> {
        &self.send
    }
}

impl<'a> PartialEq for EffectNode<'a> {
    fn eq(&self, other: &EffectNode<'a>) -> bool {
        self.id == other.id
    }
}

impl<'a> PartialOrd for EffectNode<'a> {
    fn partial_cmp(&self, other: &EffectNode<'a>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for EffectNode<'a> {
    fn cmp(&self, other: &EffectNode<'a>) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl<'a> Eq for EffectNode<'a> {}

impl<'a> Hash for EffectNode<'a> {
    fn hash<H: hash::Hasher>(&self, st: &mut H) {
        self.id.hash(st)
    }
}


impl<'a> EffectNodeBi<'a> {
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