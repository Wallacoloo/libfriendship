/// Trait for any client that wants to listen in on information that is broadcast
/// from Dispath to the *external* world. This includes notifications of state
/// change (in the routegraph), renderer results, etc.
pub trait Client {
    // TODO: use a multidimensional array type for buffer; remove num_ch param
    fn audio_rendered(&mut self, renderer_id: u32, buffer: &[f32], idx: u64, num_ch: u8) {}
}
