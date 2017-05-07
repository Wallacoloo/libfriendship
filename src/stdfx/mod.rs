//! Library of commonly-used effects - delays, filters, etc.

use routing::EffectDesc;

mod integrate;
mod passthrough;

/// Iterate over ALL the EffectDescs in the library.
pub fn iter_all_effects() -> impl Iterator<Item=EffectDesc> {
    let mut effects = Vec::new();
    effects.push(passthrough::get_desc());
    for bits in 1..64 {
        effects.push(integrate::get_desc(bits));
    }
    effects.into_iter()
}
