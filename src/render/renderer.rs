use routing::GraphWatcher;
/// Trait that allows for rendering a RouteGraph
pub trait Renderer: GraphWatcher {
    fn get_sample(&mut self, idx: u64, slot: u32) -> f32;
    // TODO: use a multidimensional array type for buffer; remove num_slots param
    /// Fill the provided buffer with samples.
    /// Samples are interleaved by slot, e.g.
    /// [t=0,s=0; t=0,s=1; t=1,s=0; t=1,s=1; t=2,s=0; ...]
    /// It's a good idea to make sure the buffer size is a multiple of the number of slots.
    fn fill_buffer(&mut self, buff: &mut [f32], idx: u64, num_slots: u32) {
        for (buff_idx, sample) in buff.iter_mut().enumerate() {
            // Note: if num_slots == 0, buff.len() == 0, so this block isn't reached.
            // Risk of division by zero is low & caused by error in the caller.
            let time_idx = idx + (buff_idx / num_slots as usize) as u64;
            let slot = (buff_idx % num_slots as usize) as u32;
            *sample = self.get_sample(time_idx, slot);
        }
    }
}
