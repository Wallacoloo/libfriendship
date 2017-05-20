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
#[derive(Debug)]
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
    fn send(&self, msg: ClientMessage) {
        trace!("Sending message to Client: {:?}", msg);
        if let Err(msg) = self.tx.send(msg) {
            warn!("Unable to send message to Client: {:?}", msg);
        }
    }
}

impl Client for MpscClient {
    fn audio_rendered(&mut self, buffer: &[f32], idx: u64, slot: u32) {
        // TODO: Clients will disconnect; can we handle this more gracefully?
        self.send(ClientMessage::AudioRendered(buffer.to_vec(), idx, slot));
    }
    fn node_meta(&mut self, handle: &NodeHandle, meta: &EffectMeta) {
        self.send(ClientMessage::NodeMeta(*handle, meta.clone()));
    }
    fn node_id(&mut self, handle: &NodeHandle, id: &EffectId) {
        self.send(ClientMessage::NodeId(*handle, id.clone()));
    }
}
