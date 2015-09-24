use super::effect::{Effect, EffectRenderState};
use partial::Partial;

pub struct Sum;

pub struct SumProcessIter {
    partial : Option<Partial>,
}

impl Iterator for SumProcessIter {
    type Item=Partial;

    fn next(&mut self) -> Option<Partial> {
        self.partial.take()
    }
}

impl Effect for Sum {
    fn new() -> Sum {
        Sum
    }
    fn process(&self, state : &mut EffectRenderState, partial : &Partial, slot_no : u32) -> Box<Iterator<Item=Partial>> {
        Box::new(SumProcessIter {partial:Some(*partial)})
    }
    fn get_input_slot(&self, index : u32) -> Option<&str> {
        match index {
            0 => Some("source0"),
            1 => Some("source1"),
            _ => None
        }
    }
}