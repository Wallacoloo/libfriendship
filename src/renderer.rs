use std::collections::BinaryHeap;
use std::cmp::PartialEq;
use std::cmp::Ordering;

use super::effect_send::EffectSend;
use super::effect_tree::EffectTree;
use super::partial::Partial;

/// Packages information on how to get the next partial in an effect's output
/// stream, where to send it, and the last retrieved partial.
/// These streams can be sorted based on pending partial's start time so that
/// partials can be handled based on how soon they must be rendered.
struct PartialStream<'a> {
    stream : Box<Iterator<Item=Partial>>,
    destination : &'a EffectSend<'a>,
    pending : Partial,
}

/// Takes an EffectTree and some partials inserted at specific locations and
/// renders a waveform (i.e. PCM signal)
pub struct Renderer<'a> {
    /// Reference to the tree that describes the connections of each effect
    tree : &'a EffectTree<'a>,
    /// Set of iterators that generate new partials packaged with information
    /// regarding where to send those partials.
    partial_streams : BinaryHeap<PartialStream<'a>>,
}


impl <'a> Ord for PartialStream<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        // we reverse the ordering so that the *soonest* Partial will always
        // appear at the tip of the heap.
        other.pending.start_time().cmp(&self.pending.start_time())
    }
}

impl <'a> PartialEq for PartialStream<'a> {
    fn eq(&self, other: &Self) -> bool {
         self.cmp(other) == Ordering::Equal
    }
}

impl <'a> Eq for PartialStream<'a> {}

impl <'a> PartialOrd for PartialStream<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        return Some(self.cmp(other))
    }

    fn lt(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Less
    }
    fn le(&self, other: &Self) -> bool {
        self.cmp(other) != Ordering::Greater
    }
    fn gt(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Greater
    }
    fn ge(&self, other: &Self) -> bool {
        self.cmp(other) != Ordering::Less
    }
}

impl <'a> Renderer <'a> {
    pub fn new(tree : &'a EffectTree<'a>) -> Renderer<'a> {
        Renderer{ tree:tree, partial_streams:BinaryHeap::new() }
    }
    pub fn advance(&mut self) -> f32 {
        0.0f32
    }
}