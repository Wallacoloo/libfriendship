/// Specifies quality, frame-rate settings, etc that are mostly static over
/// the lifetime of an audio stream
#[derive(Clone, Copy)]
pub struct RenderSpec {
    sample_rate: u32,
}

/// Factory class used to create a RenderSpec
#[derive(Clone, Copy)]
pub struct RenderSpecFactory {
    spec: RenderSpec,
}

impl RenderSpec {
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

impl RenderSpecFactory {
    pub fn new() -> RenderSpecFactory {
        RenderSpecFactory { 
            spec: RenderSpec {
                sample_rate: 0,
            }
        }
    }
    pub fn finalize(&mut self) -> RenderSpec {
        self.spec
    }
    pub fn sample_rate(&mut self, rate: u32) -> &mut RenderSpecFactory {
        self.spec.sample_rate = rate;
        self
    }
}

