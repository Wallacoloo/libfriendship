use url::Url;

use routing::{EffectMeta};

/// Metadata to invoke the primitive Delay effect.
pub fn get_meta() -> EffectMeta {
    EffectMeta::new("Delay".into(), None, [Url::parse("primitive:///Delay").unwrap()].iter().cloned())
}
