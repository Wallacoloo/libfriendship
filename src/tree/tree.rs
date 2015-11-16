use std::collections::hash_set::HashSet;
use std::rc::Rc;

use super::node::YNode;
use super::send::Send;

pub struct Tree {
    sends: HashSet<Send>,
    y_outputs: Vec<Rc<YNode>>,
}

impl Tree {
    pub fn new() -> Tree {
        Tree{ sends: HashSet::new(), y_outputs: vec![] }
    }
    pub fn add_output(&mut self, out: Rc<YNode>) {
        self.y_outputs.push(out);
    }
    pub fn add_send(&mut self, send: Send) {
        self.sends.insert(send);
    }
}