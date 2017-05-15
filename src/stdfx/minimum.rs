use url::Url;

use routing::EffectId;

/// Metadata to invoke the primitive Minimum effect.
pub fn get_id() -> EffectId {
    EffectId::new("Minimum".into(), None, [Url::parse("primitive:///Minimum").unwrap()].iter().cloned())
}
