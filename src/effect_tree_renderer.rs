use std::cmp::PartialEq;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::collections::hash_map;
use std::rc::Rc;

use super::effect::Effect;
use super::effect_node::{EffectNode, EffectNodeType};
use super::effect_send::EffectSend;
use super::effect_tree::EffectTree;
use super::partial::Partial;

pub enum StreamDest<'a> {
    EffectSends(Vec<EffectSend<'a>>),
    ChannelSink(u8),
}

/// Packages information on how to get the next partial in an effect's output
/// stream, where to send it, and the last retrieved partial.
/// These streams can be sorted based on pending partial's start time so that
/// partials can be handled based on how soon they must be rendered.
pub struct PartialStream<'a> {
    stream : EffectProcessIter,
    dest : StreamDest<'a>,
    pending : Partial,
}

/// Takes an EffectTree and some partials inserted at specific locations and
/// returns a stream of Partials outputted by the tree's root. These partials
/// can then be directly converted into a PCM/waveform signal by a separate
/// renderer.
pub struct EffectTreeRenderer<'a> {
    /// Set of iterators that generate new partials packaged with information
    /// regarding where to send those partials.
    partial_streams : BinaryHeap<PartialStream<'a>>,
    effect_states : HashMap<Rc<EffectNode<'a>>, EffectRenderState>,
}

/// State info about each node in the effect tree
enum EffectRenderState {
    /// see effect::Effect::AmpScale
    AmpScale,
    /// see effect::Effect::StartTimeOffset
    StartTimeOffset,
    /// see effect::Effect::FreqScale
    FreqScale,
    /// All inputs sent to this effect should be sent to the tree's output at a
    /// specific channel
    ChannelSink(u8),
}

/// Each partial sent to an effect creates an iterator that describes the
/// output.
pub struct EffectProcessIter {
    p : Option<Partial>,
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
    pub fn new(stream : EffectProcessIter,
      dest : StreamDest<'a>, pending : Partial)
      -> PartialStream<'a> {
        PartialStream {
            stream:stream,
            dest:dest,
            pending:pending
        }
    }
}

impl<'a> EffectTreeRenderer <'a> {
    pub fn new(tree : &'a EffectTree<'a>) -> EffectTreeRenderer<'a> {
        EffectTreeRenderer{
            partial_streams:BinaryHeap::new(),
            // Create an EffectRenderState::ChannelSink for each root of the tree
            effect_states: tree.iter_roots().enumerate().map(|(ch, root)| {
                (root.clone(), EffectRenderState::new_channel_sink(root.effect(), ch as u8))
            }).collect()
        }
    }
    /// send a partial to the given `dest`
    pub fn feed(&mut self, dest : EffectSend<'a>, partial : &Partial) {
        // send the partial to the effect, which creates an iterator for the effect's output
        let new_iter;
        let new_dests;
        {
            let mut state_entry = self.effect_states.entry(dest.effect_node().clone());
            let render_state = match state_entry {
                hash_map::Entry::Vacant(entry) => {
                    entry.insert(EffectRenderState::new(dest.effect_node().effect()))
                },
                hash_map::Entry::Occupied(ref mut entry) => entry.get_mut()
            };
            new_dests = match render_state {
                &mut EffectRenderState::ChannelSink(ref channel) => {
                    StreamDest::ChannelSink(*channel)
                },
                &mut _ => {
                    StreamDest::EffectSends(dest.effect_node().sends().clone())
                }
            };
            new_iter = render_state.process(partial, dest.send_slot());
        }
        // add the new Partial Iterator into our heap
        self.check_add_stream(new_iter, new_dests);
    }
    /// if `iter` has another item, push its next item, `dest` & `iter`
    /// onto the heap of PartialStreams
    fn check_add_stream(&mut self, mut iter : EffectProcessIter,
      dest : StreamDest<'a> ) {
        iter.next().map(|partial| {
            let stream = PartialStream::new(iter, dest, partial);
            self.partial_streams.push(stream);
        });
    }
    /// Takes the front-most Partial and processes it.
    /// Returns Some(Partial) if this results in a new Partial that has exited
    /// from the root of the tree (ready to be rendered to audio), else None.
    pub fn step(&mut self) -> Option<(u8, Partial)> {
        self.partial_streams.pop().map_or(None, |stream| {
            match stream.dest {
                StreamDest::EffectSends(sends) => {
                    // send the partial to the destination effects/slots
                    for send in &sends {
                        self.feed(send.clone(), &stream.pending);
                    }
                    // since we popped the original stream, we have to re-add it as well:
                    // NOTE: popping & re-pushing IS necessary, since by advancing the iterator,
                    // we alter this stream's position in the queue.
                    self.check_add_stream(stream.stream, StreamDest::EffectSends(sends));
                    None
                }
                // destination is an audio sink; yield the partial
                StreamDest::ChannelSink(channel) => Some((channel, stream.pending))
            }
        })
    }
}

impl EffectRenderState {
    pub fn new(effect : &EffectNodeType) -> EffectRenderState {
        match effect {
            &EffectNodeType::EffectNode(Effect::AmpScale) =>
                EffectRenderState::AmpScale,
            &EffectNodeType::EffectNode(Effect::StartTimeOffset) =>
                EffectRenderState::StartTimeOffset,
            &EffectNodeType::EffectNode(Effect::FreqScale) =>
                EffectRenderState::FreqScale,
            // Don't allow arbitrary sinks; the EffectTree must explicitly specify them.
            &EffectNodeType::Sink => panic!("EffectNodeType::Sink objects \
                must be explicitly declared by EffectTree ahead-of-time"),
        }
    }
    pub fn new_channel_sink(effect : &EffectNodeType, ch : u8) -> EffectRenderState {
        match effect {
            &EffectNodeType::Sink => EffectRenderState::ChannelSink(ch),
            _ => panic!("Tried to create an EffectRenderState::ChannelSink \
                from an EffectNode whose type was not EffectNodeType::Sink"),
        }
    }
    /// Given @partial as an input to the effect through the slot at @slot_no,
    /// returns an iterator that will enerate every future output, where each
    /// generated output's start_usec value increases monotonically.
    pub fn process(&self, partial : &Partial, _slot_no : u32) -> EffectProcessIter {
        match self {
            &EffectRenderState::AmpScale => unimplemented!(),
            &EffectRenderState::StartTimeOffset => unimplemented!(),
            &EffectRenderState::FreqScale => unimplemented!(),
            &EffectRenderState::ChannelSink(ref _channel) => EffectProcessIter{ p:Some(*partial) },
        }
    }
}

impl Iterator for EffectProcessIter {
    type Item = Partial;

    fn next(&mut self) -> Option<Partial> {
        self.p.take()
    }
}
