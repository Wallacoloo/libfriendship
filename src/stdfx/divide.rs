use url::Url;

use routing::EffectId;

/// Metadata to invoke the primitive `Divide` effect.
pub fn get_id() -> EffectId {
    EffectId::new("Divide".into(), None, [Url::parse("primitive:///Divide").unwrap()].iter().cloned())
}
