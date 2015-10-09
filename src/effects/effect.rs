use partial::Partial;

/// In order to separate the layout of the effect tree from any state info
/// mutated during the render process, effect-specific render state is stored
/// separately
pub struct EffectRenderState;

pub enum Effect {
    /// output = input * automation
    AmpScale,
    /// each signal in the input (freq=wi, start_time=t0i) is delayed by
    /// f(wi, t0i), where f(w, t0) is the automation evaluated at that
    /// freq/time (both of which are constant)
    StartTimeOffset,
    /// The frequency of each input (wi) is multiplied by f(wi, t), where
    /// f(w, t) is the automation evaluated at (w (fixed), t (variable))
    FreqScale,
}

pub struct EffectProcessIter {
    p : Option<Partial>,
}

/// An effect creates Partial outputs from a sequence of Partial inputs and
/// some extra parameters.
///
/// Effects have multiple (but a fixed number of) input slots, and exactly one
/// output.
///
/// There are a fixed number of primitive effects (enumerated further up), from
/// which all required functionality can be derived.
impl Effect {
    /// The effect's configuration is kept separately from any temporary state
    /// needed when rendering for encapsulation/immutability reasons.
    /// All state info is stored on an associated EffectRenderState.
    pub fn new_render_state(&self) -> EffectRenderState {
        EffectRenderState
    }
    /// Given @partial as an input to the effect through the slot at @slot_no,
    /// returns an iterator that will enerate every future output, where each
    /// generated output's start_usec value increases monotonically.
    pub fn process(&self, _state : &mut EffectRenderState, partial : &Partial, _slot_no : u32) -> EffectProcessIter {
        match self {
            &Effect::AmpScale => unimplemented!(),
            &Effect::StartTimeOffset => EffectProcessIter{ p:Some(*partial) },
            &Effect::FreqScale => unimplemented!(),
        }
    }
}

impl Iterator for EffectProcessIter {
    type Item = Partial;

    fn next(&mut self) -> Option<Partial> {
        self.p.take()
    }
}