use std::collections::HashSet;
use std::io::Cursor;
use std::ops::Deref;
use std::rc::Rc;

use digest::digest_reader;
use serde_json;
use sha2::Sha256;
use url::Url;
use url_serde;

use resman::ResMan;
use super::routegraph::RouteGraph;
use super::adjlist::AdjList;

#[derive(Debug)]
pub enum Error {
    /// No effect matches the metadata requested.
    NoMatchingEffect(EffectId),
}

/// Alias for a `Result` with our error type.
pub type ResultE<T> = Result<T, Error>;

/// Serializable info needed to look up an effect.
#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
pub struct EffectId {
    /// Canonical name of the effect
    name: String,
    /// Hash of the effect's definition file, or None if the effect is primitive
    sha256: Option<[u8; 32]>,
    // TODO: consider a smallset.
    // TODO: use #[serde(with = "url_serde")]
    /// List of URLs where the Effect can be obtained
    urls: HashSet<url_serde::Serde<Url>>,
}

/// All information that will be loaded from disk/network to describe how to
/// synthesize the Effect.
#[derive(Serialize, Deserialize)]
pub struct EffectDesc {
    meta: EffectMeta,
    adjlist: AdjList,
}

/// Validated version of EffectDesc. Guaranteed to be synthesizable.
pub struct Effect {
    id: EffectId,
    meta: EffectMeta,
    // TODO: Effects are immutable, so we can make this an AdjList and store
    // the connectivity information separately.
    // option, because effect MAY be primitive.
    graph: Option<RouteGraph>,
}

#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
pub struct EffectMeta {
    /// Canonical name of the effect
    name: String,
    /// List of URLs where the Effect can be obtained
    urls: HashSet<url_serde::Serde<Url>>,
}


impl Effect {
    pub fn are_slots_connected(&self, from_slot: u32, from_ch: u8, to_slot: u32, to_ch: u8) -> bool {
        match self.graph {
            Some(ref g) => g.are_slots_connected(from_slot, from_ch, to_slot, to_ch),
            // For primitive effects, we assume ALL slots are connected.
            None => true,
        }
    }
    pub fn id(&self) -> EffectId {
        EffectId {
            name: self.meta.name.clone(),
            sha256: None,
            urls: self.meta.urls.clone(),
        }
    }
    /// Given the effect's information, and an interface by which to load
    /// resources, return an actual Effect.
    pub fn from_id(id: EffectId, resman: &ResMan) -> ResultE<Rc<Self>> {
        // For primitive effects, don't attempt to locate their descriptions (they don't exist)
        if id.is_primitive() {
            let me = Self {
                meta: EffectMeta {
                    name: id.name.clone(),
                    urls: id.urls.clone(),
                },
                id: id,
                graph: None,
            };
            return Ok(Rc::new(me));
        }
        // Locate descriptions for non-primitive effects
        for reader in resman.find_effect(&id) {
            // Try to deserialize to an effect description
            let desc: Result<EffectDesc, serde_json::Error> = serde_json::from_reader(reader);
            match desc {
                Ok(desc) => {
                    // TODO: since we've matched the effect, we only need to recalculate
                    // the id if the original search had missing hashes.
                    let id = desc.id();
                    match RouteGraph::from_adjlist(desc.adjlist, resman) {
                        Ok(graph) => {
                            let me = Self {
                                id,
                                meta: desc.meta,
                                graph: Some(graph),
                            };
                            // TODO: implement some form of caching
                            return Ok(Rc::new(me));
                        },
                        Err(error) => {
                            println!("Warning: RouteGraph::from_adjlist failed: {:?}", error)
                        }
                    }
                },
                Err(error) => println!("Warning: unable to deserialize EffectDesc: {:?}", error)
            }
        }
        // No matching effects
        Err(Error::NoMatchingEffect(id))
    }
    pub fn routegraph(&self) -> &Option<RouteGraph> {
        &self.graph
    }
}

impl EffectId {
    pub fn new<U>(name: String, sha256: Option<[u8; 32]>, urls: U) -> Self 
        where U: IntoIterator<Item=Url>
    {
        Self {
            name,
            sha256,
            urls: urls.into_iter().map(|url| url_serde::Serde(url)).collect(),
        }
    }
    pub fn sha256(&self) -> &Option<[u8; 32]> {
        &self.sha256
    }
    /// Returns true if the effect cannot be decomposed.
    /// This is determined by the effect providing a SINGLE url, with the primitive:// Schema
    pub fn is_primitive(&self) -> bool {
        self.urls.len() == 1 && self.urls.iter().all(|url| {
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

impl EffectDesc {
    pub fn new(meta: EffectMeta, adjlist: AdjList) -> Self {
        Self{ meta, adjlist }
    }
    pub fn id(&self) -> EffectId {
        // TODO: calcular sha using a smaller buffer
        let as_vec = serde_json::to_vec(self).unwrap();
        let result = digest_reader::<Sha256>(&mut Cursor::new(as_vec)).unwrap();
        let mut hash: [u8; 32] = Default::default();
        hash.copy_from_slice(result.as_slice());
        EffectId {
            name: self.meta.name.clone(),
            sha256: Some(hash),
            urls: self.meta.urls.clone(),
        }
    }
}

impl EffectMeta {
    pub fn new<U>(name: String, urls: U) -> Self 
        where U: IntoIterator<Item=Url>
    {
        Self {
            name,
            urls: urls.into_iter().map(|url| url_serde::Serde(url)).collect(),
        }
    }
}

impl PartialEq for EffectMeta {
    // Equality implemented in a way where we can easily check things like
    //   "Is this the same primitive Delay effect this renderer knows how to implement?"
    fn eq(&self, other: &EffectMeta) -> bool {
        self.name == other.name &&
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

