use url::Url;

use routing::EffectId;

/// Metadata to invoke the primitive F32Constant effect.
pub fn get_id() -> EffectId {
    EffectId::new("F32Constant".into(), None, [Url::parse("primitive:///F32Constant").unwrap()].iter().cloned())
}
