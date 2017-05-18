use url::Url;

use routing::EffectId;

/// Metadata to invoke the primitive `Modulo` effect.
pub fn get_id() -> EffectId {
    EffectId::new("Modulo".into(), None, [Url::parse("primitive:///Modulo").unwrap()].iter().cloned())
}
