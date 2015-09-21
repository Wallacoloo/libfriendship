use super::effect_node::EffectNode;
use effects::effect::Effect;
use effects::sum::Sum;

pub struct EffectTree<'a> {
    root : EffectNode<'a>,
    effects : Vec<&'a Effect>,
}

impl <'a> EffectTree<'a> {
    /*pub fn new() -> EffectTree<'a> {
    	let root_effect = Box::new(Sum::new());
    	let root_node = EffectNode::new(root_effect, None);
    	EffectTree{ root:root_node, effects:vec![root_node.effect()] }
    }*/
}