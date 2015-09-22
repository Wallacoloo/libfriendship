use super::effect_node::EffectNodeBi;
use effects::effect::Effect;
use effects::sum::Sum;

pub struct EffectTree<'a> {
    root : EffectNodeBi<'a>,
}

impl <'a> EffectTree<'a> {
	/// creates a new Tree, where the root is a Sum effect
    pub fn new() -> EffectTree<'a> {
        let root_effect = Box::new(Sum::new());
        let root_node = EffectNodeBi::new(root_effect, None);
        EffectTree{ root:root_node, }
    }
    /*pub fn root(&self) -> &EffectNodeBi {
        &self.root
    }*/
}