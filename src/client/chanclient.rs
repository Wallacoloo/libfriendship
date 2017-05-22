use std::sync::mpsc::{channel, Receiver, Sender};

use ndarray::Array2;

use super::Client;
use routing::{NodeHandle, EffectMeta, EffectId};

/// Client that turns all messages into an enum variant
/// and sends them accross a thread-safe channel.
#[derive(Debug)]
pub struct MpscClient {
    tx: Sender<ClientMessage>,
}

/// Message to send across a channel, which encodes all the client
/// callback variants.
#[derive(Debug)]
pub enum ClientMessage {
    /// audio_rendered(buffer, idx, slot) call
    AudioRendered(Array2<f32>, u64),
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
    fn audio_rendered(&mut self, buffer: Array2<f32>, idx: u64) {
        self.send(ClientMessage::AudioRendered(buffer, idx));
    }
    fn node_meta(&mut self, handle: &NodeHandle, meta: &EffectMeta) {
        self.send(ClientMessage::NodeMeta(*handle, meta.clone()));
    }
    fn node_id(&mut self, handle: &NodeHandle, id: &EffectId) {
        self.send(ClientMessage::NodeId(*handle, id.clone()));
    }
}
