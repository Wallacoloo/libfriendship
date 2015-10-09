use super::partial::Partial;
use super::partial_renderer::PartialRenderer;

pub struct MultiChannelPartialRenderer {
    renderers: Vec<PartialRenderer>
}

impl MultiChannelPartialRenderer {
    pub fn new(sample_rate : u32, num_channels : u8) -> MultiChannelPartialRenderer {
        MultiChannelPartialRenderer{
            renderers: {
                (0..num_channels).map(|_i|
                    PartialRenderer::new(sample_rate)
                ).collect()
            }
        }
    }
    pub fn feed(&mut self, ch : u8, partial : Partial) {
        self.renderers[ch as usize].feed(partial)
    }
    pub fn step(&mut self, ch : u8) -> f32 {
        self.renderers[ch as usize].step()
    }
}