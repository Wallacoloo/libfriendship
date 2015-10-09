use std::cmp::Ordering;
use std::hash::{self, Hash};
use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT};
use std::sync::atomic;

use super::effect::Effect;
use super::effect_send::EffectSend;

/// This variable is incremented every time a new EffectNode is created in
/// order to give it a unique tag.
static mut effect_id_gen : AtomicUsize = ATOMIC_USIZE_INIT;

pub enum EffectNodeType {
    /// Node represents an ordinary effect
    EffectNode(Effect),
    /// Node represents some audio destination, e.g. a channel output
    Sink,
}

/// Describes a node on the effect tree, where leaves point up towards the root
/// Each leaf consists of an Effect and info on where to send that effect's
/// output.
pub struct EffectNode<'a> {
    /// in order to store this structure as a key in an associative container,
    /// e.g. a HashMap, we need some way to uniquely tag effects, hence `id`
    id : u32,
    effect : EffectNodeType,
    sends : Vec<EffectSend<'a>>,
}

impl<'a> EffectNode<'a> {
    fn next_id() -> u32 {
        let id = unsafe { effect_id_gen.fetch_add(1, atomic::Ordering::SeqCst) };
        id as u32
    }
    pub fn new(effect : Effect, sends : Vec<EffectSend<'a>>)
      -> EffectNode<'a> {
        EffectNode {
            id:EffectNode::next_id(),
            effect:EffectNodeType::EffectNode(effect),
            sends:sends
        }
    }
    pub fn new_sink() -> EffectNode<'a> {
        EffectNode {
            id:EffectNode::next_id(),
            effect:EffectNodeType::Sink,
            sends:vec![]
        }
    }
    pub fn effect(&self) -> &EffectNodeType {
        &self.effect
    }
    pub fn sends(&self) -> &Vec<EffectSend<'a>> {
        &self.sends
    }
    pub fn add_send(&mut self, send : EffectSend<'a>) {
        self.sends.push(send.clone());
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
