use routing::GraphWatcher;
/// Trait that allows for rendering a RouteGraph
pub trait Renderer: GraphWatcher {
    fn get_sample(&mut self, idx: u64, ch: u8) -> f32;
    // TODO: use a multidimensional array type for buffer; remove num_ch param
    /// Fill the provided buffer with samples.
    /// Samples are interleaved by channel, e.g.
    /// [t=0,c=0; t=0,c=1; t=1,c=0; t=1,c=1; t=2,c=0; ...]
    /// It's a good idea to make sure the buffer size is a multiple of the number of channels.
    fn fill_buffer(&mut self, buff: &mut [f32], idx: u64, num_ch: u8) {
        for (buff_idx, sample) in buff.iter_mut().enumerate() {
            let time_idx = (buff_idx / num_ch as usize) as u64;
            let ch = (buff_idx % num_ch as usize) as u8;
            *sample = self.get_sample(time_idx, ch);
        }
    }
}
