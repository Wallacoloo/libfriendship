//! Library of commonly-used effects - delays, filters, etc.

use routing::EffectDesc;

// Thin wrappers around primitive types;
// they have no actual EffectDescs - just EffectIds
mod delay;
mod divide;
mod f32constant;
mod minimum;
mod modulo;
mod multiply;

mod integrate;
mod fir;
mod hamming;
mod modulo_one;
mod passthrough;
mod unitsaw;

/// Iterate over ALL the EffectDescs in the library.
pub fn iter_all_effects() -> impl Iterator<Item=EffectDesc> {
    let effects = None.into_iter();

    // Passthrough (i.e. NOOP)
    let effects = effects.chain(Some(passthrough::get_desc()).into_iter());
    // Modulo by 1.0
    let effects = effects.chain(Some(modulo_one::get_desc()).into_iter());

    // Integrate
    let effects = effects.chain((1..65).map(|bits| {
        integrate::get_desc(bits)
    }));

    // Finite Impulse Response
    let effects = effects.chain((1..16).map(|bits| {
        fir::get_desc(1 << bits)
    }));

    // Windowing function: Hamming
    let effects = effects.chain((2..513).map(|n| {
        hamming::get_desc(n)
    }));

    // Oscillator function: Sawtooth
    let effects = effects.chain(Some(unitsaw::get_desc()).into_iter());

    effects
}
