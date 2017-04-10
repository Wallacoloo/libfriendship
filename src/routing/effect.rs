use serde_json::map::Map;
use serde_json::value::Value;

use super::routegraph::RouteGraph;

/// Serializable info needed to look up an effect.
#[derive(Clone)]
#[derive(Serialize, Deserialize)]
pub struct EffectDesc {
    /// Canonical name of the effect
    name: String,
    /// Hash of the effect's definition file, or None if the effect is primitive
    sha256: Option<[u8; 32]>,
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
        match self.graph {
            Some(ref g) => g.are_slots_connected(from_slot, from_ch, to_slot, to_ch),
            // For primitive effects, we assume ALL slots are connected.
            None => true,
        }
    }
    pub fn desc(&self) -> EffectDesc {
        self.desc.clone()
    }
}

impl EffectDesc {
    pub fn sha256(&self) -> &Option<[u8; 32]> {
        &self.sha256
    }
}

impl PartialEq for EffectDesc {
    // Equality implemented in a way where we can easily check things like
    //   "Is this the same primitive Delay effect this renderer knows how to implement?"
    fn eq(&self, other: &EffectDesc) -> bool {
        self.name == other.name &&
            self.sha256 == other.sha256 &&
            self.effect_args == other.effect_args
    }
}
impl Eq for EffectDesc {}

impl PartialEq for Effect {
    fn eq(&self, other: &Effect) -> bool {
        self.desc == other.desc
    }
}
impl Eq for Effect {}
