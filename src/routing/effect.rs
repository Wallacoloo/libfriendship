use std::rc::Rc;

use serde_json;
use serde_json::map::Map;
use serde_json::value::Value;

use resman::ResMan;
use super::routegraph::RouteGraph;
use super::adjlist::AdjList;

/// Serializable info needed to look up an effect.
#[derive(Clone)]
#[derive(Serialize, Deserialize)]
pub struct EffectMeta {
    /// Canonical name of the effect
    name: String,
    /// Hash of the effect's definition file, or None if the effect is primitive
    sha256: Option<[u8; 32]>,
    /// List of URLs where the Effect can be obtained
    urls: Vec<String>,
    /// Arguments specific to the given effect (relevant for e.g. Constant).
    effect_args: Map<String, Value>,
}

/// All information that will be loaded from disk/network to describe how to
/// synthesize the Effect.
#[derive(Serialize, Deserialize)]
struct EffectDesc {
    // TODO: when de/serializing, the hashes should be removed from EffectMeta
    meta: EffectMeta,
    adjlist: AdjList,
}

/// Validated version of EffectDesc. Guaranteed to be synthesizable.
pub struct Effect {
    meta: EffectMeta,
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
    pub fn meta(&self) -> EffectMeta {
        self.meta.clone()
    }
    /// Given the effect's information, and an interface by which to load
    /// resources, return an actual Effect.
    pub fn from_meta(meta: EffectMeta, resman: &ResMan) -> Result<Rc<Self>, ()> {
        for reader in resman.find_effect(&meta) {
            // Try to deserialize to an effect description
            let desc: Result<EffectDesc, serde_json::Error> = serde_json::from_reader(reader);
            if let Ok(desc) = desc {
                // Validate the data
                if let Ok(graph) = RouteGraph::from_adjlist(desc.adjlist, resman) {
                    let me = Self {
                        meta: desc.meta,
                        graph: Some(graph),
                    };
                    return Ok(Rc::new(me));
                }
            }
        }
        // No matching effects
        Err(())
    }
}

impl EffectMeta {
    pub fn sha256(&self) -> &Option<[u8; 32]> {
        &self.sha256
    }
}

impl PartialEq for EffectMeta {
    // Equality implemented in a way where we can easily check things like
    //   "Is this the same primitive Delay effect this renderer knows how to implement?"
    fn eq(&self, other: &EffectMeta) -> bool {
        self.name == other.name &&
            self.sha256 == other.sha256 &&
            self.effect_args == other.effect_args
    }
}
impl Eq for EffectMeta {}

impl PartialEq for Effect {
    fn eq(&self, other: &Effect) -> bool {
        self.meta == other.meta
    }
}
impl Eq for Effect {}
