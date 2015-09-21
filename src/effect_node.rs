use effects::effect::Effect;
use super::effect_send::EffectSend;

use std::ops::Deref;

pub struct EffectNode<'a> {
    effect : Box<Effect>,
    send : Option<EffectSend<'a>>,
}

impl <'a> EffectNode<'a> {
    pub fn new(effect : Box<Effect>, send : Option<EffectSend<'a>>)
      -> EffectNode<'a> {
        EffectNode{ effect:effect, send:send }
    }
    pub fn effect(&'a self) -> &'a Effect {
    	self.effect.deref()
    }
}