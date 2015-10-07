use std::rc::Rc;

use super::effect_node::EffectNode;
use effects::effect::Effect;
use effects::sum::Sum;

pub struct EffectTree<'a> {
    root : Rc<EffectNode<'a>>,
}

impl<'a> EffectTree<'a> {
    /// creates a new Tree, where the root is a Sum effect
    pub fn new() -> EffectTree<'a> {
        let root_effect = Box::new(Sum::new());
        let root_node = Rc::new(EffectNode::new(root_effect, vec![]));
        EffectTree{ root:root_node }
    }
    pub fn root(&'a self) -> &Rc<EffectNode> {
        &self.root
    }
}
