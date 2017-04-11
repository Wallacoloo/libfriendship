/// Trait that allows for rendering a RouteGraph
pub trait Renderer {
    fn get_sample(&mut self, idx: u64) -> f32;
}
