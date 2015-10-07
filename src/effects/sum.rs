use std::iter;

use super::effect::{Effect, EffectRenderState};
use partial::Partial;

/// The output for a Sum effect is the sum of all partials sent to any of its
/// slots.
pub struct Sum;

impl Effect for Sum {
    fn new() -> Sum {
        Sum
    }
    fn process(&self, _state : &mut EffectRenderState, partial : &Partial, _slot_no : u32)
      -> Box<Iterator<Item=Partial>> {
        // return an iterator that will provide Some(*partial) upon the first
        // call and None upon any future calls
        Box::new(iter::once(*partial))
    }
    fn get_input_slot(&self, index : u32) -> Option<&str> {
        match index {
            0 => Some("source0"),
            1 => Some("source1"),
            _ => None
        }
    }
}