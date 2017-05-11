use url::Url;

use routing::{EffectMeta};

/// Metadata to invoke the primitive F32Constant effect.
pub fn get_meta() -> EffectMeta {
    EffectMeta::new("F32Constant".into(), None, [Url::parse("primitive:///F32Constant").unwrap()].iter().cloned())
}
