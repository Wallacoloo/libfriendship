/// Specifies quality settings, etc that are mostly static over
/// the lifetime of an audio stream
#[derive(Clone, Copy)]
pub struct RenderSpec {
    // target latency, in samples.
    max_latency: u32,
}

impl RenderSpec {
    pub fn max_latency(&self) -> u32 {
        self.max_latency
    }
}


impl Default for RenderSpec {
    fn default() -> RenderSpec {
        RenderSpec {
            max_latency: 256,
        }
    }
}
