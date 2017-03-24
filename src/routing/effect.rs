// TODO: rust-crypto lib for SHA256 verification.

extern crate serde;
extern crate serde_json;
use self::serde_json::map::Map;
use self::serde_json::value::Value;

#[derive(Serialize, Deserialize)]
pub struct Effect {
    /// Canonical name of the effect
    name: String,
    /// Hash of the effect's definition file, or None if the effect is primitive
    sha1: Option<[u8; 32]>,
    /// List of URLs where the Effect can be obtained
    urls: Vec<String>,
    /// Arguments specific to the given effect (relevant for e.g. Constant).
    effect_args: Map<String, Value>,
}

