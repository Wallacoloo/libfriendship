use url::Url;

use routing::{EffectMeta};

/// Metadata to invoke the primitive Divide effect.
pub fn get_meta() -> EffectMeta {
    EffectMeta::new("Divide".into(), None, [Url::parse("primitive:///Divide").unwrap()].iter().cloned())
}
