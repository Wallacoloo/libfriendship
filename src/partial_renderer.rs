use super::partial::Partial;

/// Takes a series of Partials and turns them intoa PCM/audio signal.
pub struct PartialRenderer;

impl PartialRenderer {
	pub fn new() -> PartialRenderer {
		PartialRenderer
	}
	pub fn feed(&mut self, partial : &Partial) {
		unimplemented!();
	}
	pub fn step(&mut self) -> f32 {
		unimplemented!();
	}
}