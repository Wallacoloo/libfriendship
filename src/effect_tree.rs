use std::rc::Rc;
use std::slice;

use super::effect_node::EffectNode;

pub struct EffectTree<'a> {
    roots : Vec<Rc<EffectNode<'a>>>,
}

impl<'a> EffectTree<'a> {
    /// creates a new Tree with @num_channels roots
    pub fn new(num_channels : u8) -> EffectTree<'a> {
        let mut roots = vec![];
        for _ in 0..num_channels {
            roots.push(Rc::new(EffectNode::new_sink()));
        }
        EffectTree{ roots:roots }
    }
    /// access the @n'th root (channel) of the tree
    pub fn root(&self, n : u8) -> &Rc<EffectNode<'a>> {
        &self.roots[n as usize]
    }
    pub fn iter_roots(&'a self) -> slice::Iter<'a, Rc<EffectNode<'a>>> {
        self.roots.iter()
    }
}
