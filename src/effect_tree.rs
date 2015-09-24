use super::effect_node::{EffectNode, EffectNodeBi};
use effects::effect::Effect;
use effects::sum::Sum;

pub struct EffectTree<'a> {
    root : EffectNodeBi<'a>,
}

pub struct EffectTreeIter<'a> {
    // Given a tree like this (a is root):
    //
    //  e      f g
    //  |      |/
    //  b      c      d
    //   \     |     /
    //    -----a-----
    // Path starts as (a, 0)
    // Iteration proceeds as:
    // [(a, 0), (b, 0)]
    // [(a, 0), (b, 0), (e, 0)]
    // [(a, 1), (c, 0)]
    // [(a, 1), (c, 0), (f, 0)]
    // [(a, 1), (c, 1), (g, 0)]
    // [(a, 2), (d, 0)]
    // []
    // Thus to advance the iterator, we look at the last part of the path (node,idx):
    //   check if there exists a node at node.children[idx]
    //     if so, append (node.children[idx], 0) to path
    //   else, pop the last path, increment the index in the new path.last() and repeat
    // Example:
    // [(a, 0), (b, 0)]: yes - b has a child at index 0.
    //   therefore push (e, 0) onto the path
    //   new path = [(a, 0), (b, 0), (e, 0)]
    // [(a, 0), (b, 0), (e, 0)]: no - e has no child at index 0.
    //   pop (e, 0) and increment the index of the new path end:
    //   [(a, 0), (b, 1)]
    //   repeat: does b have a child at index 1? no
    //   pop (b, 1) and increment the index of the new path end:
    //   [(a, 1)]
    //   repeat: does a have a child at index 1? yes
    //   therefore, push (c, 0) onto the path
    //   new path: [(a, 1), (c, 0)]
    path : Vec<(&'a EffectNodeBi<'a>, usize)>,
}

impl<'a> EffectTree<'a> {
    /// creates a new Tree, where the root is a Sum effect
    pub fn new() -> EffectTree<'a> {
        let root_effect = Box::new(Sum::new());
        let root_node = EffectNodeBi::new(root_effect, None);
        EffectTree{ root:root_node }
    }
    /// Iterate the tree in a totally unspecified order.
    /// The primary use of this function is just to obtain a set of all
    /// EffectNodes
    pub fn iter(&'a self) -> EffectTreeIter<'a> {
        EffectTreeIter{ path:vec![(&self.root, 0)] }
    }
    pub fn root(&self) -> &EffectNodeBi {
        &self.root
    }
}

impl<'a> Iterator for EffectTreeIter<'a> {
    type Item=&'a EffectNode<'a>;

    fn next(&mut self) -> Option<&'a EffectNode<'a>> {
        let ret = self.path.last().map(|last| last.0.node());
        // Walk the tree until we find the next node, or reach its end
        while let Some(active_node) = self.path.last().cloned() {
            // if the next node in the tree is valid, then append that and break.
            // otherwise, walk back up a step and move one child right
            match active_node.0.child_node_bi(active_node.1) {
                Some(node) => {
                    self.path.push((node, 0));
                    break;
                },
                None => {
                    self.path.pop();
                    self.path.last_mut().map(|new_last| {
                        new_last.1 += 1
                    });
                }
            }
        }
        ret
    }
}
