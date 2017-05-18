use std::sync::mpsc::{channel, Receiver, Sender};

use super::Client;
use routing::{NodeHandle, EffectMeta, EffectId};

/// Client that turns all messages into an enum variant
/// and sends them accross a thread-safe channel.
pub struct MpscClient {
    tx: Sender<ClientMessage>,
}

/// Message to send across a channel, which encodes all the client
/// callback variants.
pub enum ClientMessage {
    /// audio_rendered(buffer, idx, slot) call
    AudioRendered(Vec<f32>, u64, u32),
    /// node_meta(handle, meta) call
    NodeMeta(NodeHandle, EffectMeta),
    /// node_id(handle, id) call
    NodeId(NodeHandle, EffectId),
}

impl MpscClient {
    pub fn new() -> (Self, Receiver<ClientMessage>) {
        let (tx, rx) = channel();
        (Self{ tx }, rx)
    }
}

impl Client for MpscClient {
    fn audio_rendered(&mut self, buffer: &[f32], idx: u64, slot: u32) {
        // TODO: Clients will disconnect; can we handle this more gracefully?
        self.tx.send(ClientMessage::AudioRendered(buffer.to_vec(), idx, slot)).expect("Unable to reach Client");
    }
    fn node_meta(&mut self, handle: &NodeHandle, meta: &EffectMeta) {
        self.tx.send(ClientMessage::NodeMeta(*handle, meta.clone())).expect("Unable to reach Client");
    }
    fn node_id(&mut self, handle: &NodeHandle, id: &EffectId) {
        self.tx.send(ClientMessage::NodeId(*handle, id.clone())).expect("Unable to reach Client");
    }
}
