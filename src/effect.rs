use partial::Partial;

/// An effect creates Partial outputs from a sequence of Partial inputs and
/// some extra parameters.
///
/// Effects have multiple (but a fixed number of) input slots, and exactly one
/// output (which may potentially be routed to multiple places).
///
/// There are a fixed number of primitive effects (enumerated below) from
/// which all required functionality can be derived.
#[derive(Clone)]
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
