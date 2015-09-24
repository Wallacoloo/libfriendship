use std::cmp::PartialEq;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;

use effects::effect::{Effect, EffectRenderState};
use super::effect_node::EffectNode;
use super::effect_send::EffectSend;
use super::effect_tree::EffectTree;
use super::partial::Partial;

/// Packages information on how to get the next partial in an effect's output
/// stream, where to send it, and the last retrieved partial.
/// These streams can be sorted based on pending partial's start time so that
/// partials can be handled based on how soon they must be rendered.
pub struct PartialStream<'a> {
    stream : Box<Iterator<Item=Partial>>,
    destination : Option<EffectSend<'a>>,
    pending : Partial,
}

/// Takes an EffectTree and some partials inserted at specific locations and
/// returns a stream of Partials outputted by the tree's root. These partials
/// can then be directly converted into a PCM/waveform signal by a separate
/// renderer.
pub struct EffectTreeRenderer<'a> {
    /// Reference to the tree that describes the connections of each effect
    tree : &'a EffectTree<'a>,
    /// Set of iterators that generate new partials packaged with information
    /// regarding where to send those partials.
    partial_streams : BinaryHeap<PartialStream<'a>>,
    effect_states : HashMap<&'a EffectNode<'a>, EffectRenderState>,
}


impl<'a> Ord for PartialStream<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        // we reverse the ordering so that the *soonest* Partial will always
        // appear at the tip of the heap.
        other.pending.start_time().cmp(&self.pending.start_time())
    }
}

impl<'a> PartialEq for PartialStream<'a> {
    fn eq(&self, other: &Self) -> bool {
         self.cmp(other) == Ordering::Equal
    }
}

impl<'a> Eq for PartialStream<'a> {}

impl<'a> PartialOrd for PartialStream<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        return Some(self.cmp(other))
    }
}

impl<'a> PartialStream<'a> {
    pub fn new(stream : Box<Iterator<Item=Partial>>,
      destination : Option<EffectSend<'a>>, pending : Partial)
      -> PartialStream<'a> {
          PartialStream{ stream:stream, destination:destination, pending:pending }
    }
}

impl<'a> EffectTreeRenderer <'a> {
    pub fn new(tree : &'a EffectTree<'a>) -> EffectTreeRenderer<'a> {
        EffectTreeRenderer{
            tree:tree,
            partial_streams:BinaryHeap::new(),
            // Create an EffectRenderState for each node in the tree
            effect_states:tree.iter().map(|node| (node, node.effect().new_render_state())).collect()
        }
    }
    /// send a partial to the given `dest`
    pub fn feed(&mut self, dest : EffectSend<'a>, partial : &Partial) {
        // send the partial to the effect, which creates an iterator for the effect's output
        let new_iter = dest.send(
            self.effect_states.get_mut(dest.effect_node()).unwrap(),
            partial);
        // add the new Partial Iterator into our heap
        self.check_add_stream(new_iter, *dest.effect_node().effect_send());
    }
    /// if `iter` has another item, push its next item, `destination` & `iter`
    /// onto the heap of PartialStreams
    fn check_add_stream(&mut self, mut iter : Box<Iterator<Item=Partial>>,
      destination : Option<EffectSend<'a>> ) {
        iter.next().map(|partial| {
            let stream = PartialStream::new(iter, destination, partial);
            self.partial_streams.push(stream);
        });
    }
    /// Takes the front-most Partial and processes it.
    /// Returns Some(Partial) if this results in a new Partial that has exited
    /// from the root of the tree (ready to be rendered to audio), else None.
    pub fn step(&mut self) -> Option<Partial> {
        self.partial_streams.pop().map_or(None, |stream| {
            match stream.destination {
                // destination=None means this partial has been fully processed
                None => Some(stream.pending),
                Some(effect_send) => {
                    // send the partial to the destination effect/slot
                    self.feed(effect_send, &stream.pending);
                    // since we popped the original stream, we have to re-add it as well:
                    self.check_add_stream(stream.stream, Some(effect_send));
                    None
                }
            }
        })
    }
}
