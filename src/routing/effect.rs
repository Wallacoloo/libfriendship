use std::collections::HashSet;
use std::ops::Deref;
use std::rc::Rc;

use serde_json;
use url::Url;
use url_serde;

use resman::ResMan;
use super::routegraph::RouteGraph;
use super::adjlist::AdjList;

#[derive(Debug)]
pub enum Error {
    /// No effect matches the metadata requested.
    NoMatchingEffect,
}

/// Alias for a `Result` with our error type.
pub type ResultE<T> = Result<T, Error>;

/// Serializable info needed to look up an effect.
#[derive(Clone)]
#[derive(Serialize, Deserialize)]
pub struct EffectMeta {
    /// Canonical name of the effect
    name: String,
    /// Hash of the effect's definition file, or None if the effect is primitive
    sha256: Option<[u8; 32]>,
    // TODO: consider a smallset.
    /// List of URLs where the Effect can be obtained
    urls: HashSet<url_serde::Serde<Url>>,
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
    // TODO: Effects are immutable, so we can make this an AdjList and store
    // the connectivity information separately.
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
    pub fn meta(&self) -> &EffectMeta {
        &self.meta
    }
    /// Given the effect's information, and an interface by which to load
    /// resources, return an actual Effect.
    pub fn from_meta(meta: EffectMeta, resman: &ResMan) -> ResultE<Rc<Self>> {
        // For primitive effects, don't attempt to locate their descriptions (they don't exist)
        if meta.is_primitive() {
            let me = Self {
                meta: meta,
                graph: None,
            };
            return Ok(Rc::new(me));
        }
        // Locate descriptions for non-primitive effects
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
                    // TODO: implement some form of caching
                    return Ok(Rc::new(me));
                }
            }
        }
        // No matching effects
        Err(Error::NoMatchingEffect)
    }
}

impl EffectMeta {
    pub fn sha256(&self) -> &Option<[u8; 32]> {
        &self.sha256
    }
    /// Returns true if the effect cannot be decomposed.
    /// This is determined by the effect providing a SINGLE url, with the primitive:// Schema
    pub fn is_primitive(&self) -> bool {
        !self.urls.len() == 1 && self.urls.iter().all(|url| {
            url.scheme() == "primitive"
        })
    }
    pub fn get_primitive_url(&self) -> Option<&Url> {
        if self.is_primitive() {
            self.urls.iter().next().map(|url| url.deref())
        } else {
            None
        }
    }
}

impl PartialEq for EffectMeta {
    // Equality implemented in a way where we can easily check things like
    //   "Is this the same primitive Delay effect this renderer knows how to implement?"
    fn eq(&self, other: &EffectMeta) -> bool {
        self.name == other.name &&
            self.sha256 == other.sha256 &&
            self.urls == other.urls
    }
}
impl Eq for EffectMeta {}

impl PartialEq for Effect {
    fn eq(&self, other: &Effect) -> bool {
        self.meta == other.meta
    }
}
impl Eq for Effect {}
