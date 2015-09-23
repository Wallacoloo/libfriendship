use partial::Partial;

/// In order to separate the layout of the effect tree from any state info
/// mutated during the render process, effect-specific render state (userdata)
/// is stored separately
pub struct EffectRenderState;

/// An effect creates Partial outputs from a sequence of Partial inputs and
/// some extra parameters.
///
/// Effects have multiple (but a fixed number of) input slots, and exactly one
/// output.
///
/// An example effect is a "Delay" effect, which just delays the signal by some
/// length of time, or an "Equalize" effect, which scales the amplitude of each
/// input Partial based upon a function of its frequency.
///
/// Another example is a "Sum" effect, which has two slots: "sig0", "sig1" and
/// outputs the sum of all waves sent to either slot.
pub trait Effect {
    fn new() -> Self where Self : Sized;
    /// Given @partial as an input to the effect through the slot at @slot_no,
    /// returns an iterator that will enerate every future output, where each
    /// generated output's start_usec value increases monotonically.
    fn process(&self, state : &mut EffectRenderState, partial : &Partial, slot_no : u32) -> Box<Iterator<Item=Partial>>;
    /// Returns information (currently just the friendly label) about the slot
    /// at index @index, or None if the slot doesn't exist.
    /// Slots are not sparse, so the lowest index for which get_input_slot
    /// returns None also represents the total number of slots.
    fn get_input_slot(&self, index : u32) -> Option<&str>;
}
