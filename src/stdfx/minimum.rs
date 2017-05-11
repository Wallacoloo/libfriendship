use url::Url;

use routing::{EffectMeta};

/// Metadata to invoke the primitive Minimum effect.
pub fn get_meta() -> EffectMeta {
    EffectMeta::new("Minimum".into(), None, [Url::parse("primitive:///Minimum").unwrap()].iter().cloned())
}
