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
    /// e.g. a HashMap, we need some way to uniquely tag effects, hence `id`
    id : u32,
    effect : Effect,
    sends : Vec<EffectSend<'a>>,
}

impl<'a> EffectNode<'a> {
    pub fn new(effect : Effect, sends : Vec<EffectSend<'a>>)
      -> EffectNode<'a> {
        let id = unsafe { effect_id_gen.fetch_add(1, atomic::Ordering::SeqCst) };
        EffectNode{ effect:effect, sends:sends, id:id as u32 }
    }
    pub fn effect(&self) -> &Effect {
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
