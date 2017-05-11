use url::Url;

use routing::{EffectMeta};

/// Metadata to invoke the primitive Multiply effect.
pub fn get_meta() -> EffectMeta {
    EffectMeta::new("Multiply".into(), None, [Url::parse("primitive:///Multiply").unwrap()].iter().cloned())
}
