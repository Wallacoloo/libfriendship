use partial::Partial;
use super::partial_renderer::PartialRenderer;

/// Serves as a simple adapter to render N channels of audio
/// using the mono-channel `partial_renderer`
pub struct NChannelPartialRenderer {
    renderers: Vec<PartialRenderer>
}

impl NChannelPartialRenderer {
    pub fn new(sample_rate : u32, num_channels : u8) -> NChannelPartialRenderer {
        NChannelPartialRenderer{
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