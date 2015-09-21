use super::effect::Effect;
use partial::Partial;

pub struct Sum;

pub struct SumProcessIter {
    partial : Partial,
    iter_complete : bool,
}

impl Iterator for SumProcessIter {
    type Item=Partial;

    fn next(&mut self) -> Option<Partial> {
        match self.iter_complete {
            false => { self.iter_complete=true; Some(self.partial) }
            true => None
        }
    }
}

impl Effect for Sum {
    fn new() -> Sum {
        Sum
    }
    fn process(&mut self, partial : &Partial, slot_no : u32) -> Box<Iterator<Item=Partial>> {
        Box::new(SumProcessIter {partial:*partial, iter_complete:false})
    }
    fn get_input_slot(&self, index : u32) -> Option<&str> {
        match index {
            0 => Some("source0"),
            1 => Some("source1"),
            _ => None
        }
    }
}