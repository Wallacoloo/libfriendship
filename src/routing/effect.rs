// TODO: rust-crypto lib for SHA256 verification.

extern crate serde;
extern crate serde_json;
use self::serde_json::map::Map;
use self::serde_json::value::Value;

use super::routegraph::RouteGraph;

#[derive(Serialize, Deserialize)]
/// Serializable info needed to look up an effect.
pub struct EffectDesc {
    /// Canonical name of the effect
    name: String,
    /// Hash of the effect's definition file, or None if the effect is primitive
    sha1: Option<[u8; 32]>,
    /// List of URLs where the Effect can be obtained
    urls: Vec<String>,
    /// Arguments specific to the given effect (relevant for e.g. Constant).
    effect_args: Map<String, Value>,
}

/// All information needed to synthesize the effect.
pub struct Effect {
    desc: EffectDesc,
    // option, because effect MAY be primitive.
    graph: Option<RouteGraph>,
}

impl Effect {
    pub fn are_slots_connected(&self, from_slot: u32, from_ch: u8, to_slot: u32, to_ch: u8) -> bool {
        unimplemented!();
    }
}
