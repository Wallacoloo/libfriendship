use routing::GraphWatcher;
/// Trait that allows for rendering a RouteGraph
pub trait Renderer: GraphWatcher {
    fn get_sample(&mut self, idx: u64, slot: u32) -> f32;
    // TODO: use a multidimensional array type for buffer; remove num_slots param
    /// Fill the provided buffer with samples from a specific slot.
    fn fill_buffer(&mut self, buff: &mut [f32], idx: u64, slot: u32) {
        for (buff_idx, sample) in buff.iter_mut().enumerate() {
            let time_idx = idx + buff_idx as u64;
            *sample = self.get_sample(time_idx, slot);
        }
    }
}
