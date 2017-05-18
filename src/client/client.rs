use routing::{NodeHandle, EffectMeta, EffectId};

/// Trait for any client that wants to listen in on information that is broadcast
/// from Dispath to the *external* world. This includes notifications of state
/// change (in the routegraph), renderer results, etc.
pub trait Client {
    /// Audio has been produced from the toplevel DAG
    fn audio_rendered(&mut self, _buffer: &[f32], _idx: u64, _slot: u32) {}
    /// Response to a query of a node's metadata
    fn node_meta(&mut self, _handle: &NodeHandle, _meta: &EffectMeta) {}
    /// Response to a query of a node's id
    fn node_id(&mut self, _handle: &NodeHandle, _id: &EffectId) {}
}
