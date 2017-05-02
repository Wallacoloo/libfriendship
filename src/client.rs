/// Trait for any client that wants to listen in on information that is broadcast
/// from Dispath to the *external* world. This includes notifications of state
/// change (in the routegraph), renderer results, etc.
pub trait Client {
    // TODO: use a multidimensional array type for buffer; remove num_ch param
    fn audio_rendered(&mut self, _renderer_id: u32, _buffer: &[f32], _idx: u64, _num_ch: u8) {}
}
