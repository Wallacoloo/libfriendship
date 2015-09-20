use partial::Partial;

/// An effect creates Partial outputs from a sequence of Partial inputs and
/// some extra parameters.
///
/// An example effect is a "Delay" effect, which just delays the signal by some
/// length of time, or an "Equalize" effect, which scales the amplitude of each
/// input Partial based upon a function of its frequency.
pub trait Effect {
    /// given @partial as an input to the effect, returns an iterator that will
    /// generate every future output, where each generated output's start_usec
    /// value increases monotonically.
    fn process(partial : &Partial) -> Box<Iterator<Item=Partial>>;
}