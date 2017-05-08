//! Library of commonly-used effects - delays, filters, etc.

use routing::EffectDesc;

mod convolve;
mod integrate;
mod passthrough;

/// Iterate over ALL the EffectDescs in the library.
pub fn iter_all_effects() -> impl Iterator<Item=EffectDesc> {
    let effects = Some(passthrough::get_desc()).into_iter();

    // Integrate
    let effects = effects.chain((1..65).map(|bits| {
        integrate::get_desc(bits)
    }));

    // Convolve
    let effects = effects.chain((1..16).map(|bits| {
        convolve::get_desc(1 << bits)
    }));

    effects
}
