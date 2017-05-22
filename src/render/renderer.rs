use jagged_array::Jagged2;
use ndarray::Array2;

use routing::GraphWatcher;
/// Trait that allows for rendering a `RouteGraph`
pub trait Renderer: GraphWatcher {
    /// Fill the provided buffer with samples from a specific slot.
    /// First, `inputs[0]` is fed to slot=0, `inputs[1]` to slot=1, and so forth.
    /// Then `buff[[0, ..]]` is filled with the output of slot=0,
    /// `buff[[1, ..]]` is filled with slot=1, and so forth.
    /// 
    /// Note that if `idx` != to 1 + the last index queried, then this is considered
    /// a "seeking" operation and the renderer is expected to flush all its internal state;
    /// i.e. it should act as if the inputs into all slots were 0 for all times outside
    /// the range being queried.
    fn fill_buffer(&mut self, buff: &mut Array2<f32>, idx: u64, inputs: Jagged2<f32>);
}
