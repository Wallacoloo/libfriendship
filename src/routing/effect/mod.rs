/// Provides functionality to describe a more abstracted view of the audio signal flow
/// as may be interacted with by a human.
///
/// Effects bundle mathematical constraints on the sample values into something more usable
///   (e.g. delay the signal by X, or low-pass filter at frequency Y with resonance Q).
/// Notably, channels are routed homogeneously through an EffectGraph. i.e. each Effect operates
/// on atomic inputs/outputs that package each channel adjacently.

mod effect;
mod effectgraph;
