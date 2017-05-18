use url::Url;

use routing::EffectId;

/// Metadata to invoke the primitive `Delay` effect.
pub fn get_id() -> EffectId {
    EffectId::new("Delay".into(), None, [Url::parse("primitive:///Delay").unwrap()].iter().cloned())
}
