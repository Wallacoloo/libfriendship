use std::sync::mpsc::{channel, Receiver, Sender};

use super::Client;
use routing::{NodeHandle, EffectMeta};

/// Client that turns all messages into an enum variant
/// and sends them accross a thread-safe channel.
pub struct MpscClient {
    tx: Sender<ClientMessage>,
}

/// Message to send across a channel, which encodes all the client
/// callback variants.
pub enum ClientMessage {
    /// audio_rendered(buffer, idx, num_slots) call
    AudioRendered(Vec<f32>, u64, u32),
    /// node_queried(handle, meta) call
    NodeQueried(NodeHandle, EffectMeta),
}

impl MpscClient {
    pub fn new() -> (Self, Receiver<ClientMessage>) {
        let (tx, rx) = channel();
        (Self{ tx }, rx)
    }
}

impl Client for MpscClient {
    fn audio_rendered(&mut self, buffer: &[f32], idx: u64, num_slots: u32) {
        self.tx.send(ClientMessage::AudioRendered(buffer.to_vec(), idx, num_slots));
    }
    fn node_queried(&mut self, handle: &NodeHandle, meta: &EffectMeta) {
        self.tx.send(ClientMessage::NodeQueried(handle.clone(), meta.clone()));
    }
}
