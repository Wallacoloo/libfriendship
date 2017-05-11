use url::Url;

use routing::{EffectMeta};

/// Metadata to invoke the primitive Modulo effect.
pub fn get_meta() -> EffectMeta {
    EffectMeta::new("Modulo".into(), None, [Url::parse("primitive:///Modulo").unwrap()].iter().cloned())
}
