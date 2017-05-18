use url::Url;

use routing::EffectId;

/// Metadata to invoke the primitive `Multiply` effect.
pub fn get_id() -> EffectId {
    EffectId::new("Multiply".into(), None, [Url::parse("primitive:///Multiply").unwrap()].iter().cloned())
}
