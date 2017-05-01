/// This serves as the "reference" audio renderer.
/// i.e. it aims to be simple and easy to understand, with little care
/// towards resource usage.

mod renderer;
pub use self::renderer::RefRenderer;
