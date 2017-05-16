use std::collections::{HashMap, HashSet};
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
    data: EffectData,
}

#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
pub struct EffectMeta {
    /// Canonical name of the effect
    name: String,
    /// List of URLs where the Effect can be obtained
    urls: HashSet<url_serde::Serde<Url>>,
    inputs: HashMap<u32, EffectInput>,
    outputs: HashMap<u32, EffectInput>,
}

#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
pub struct EffectIO {
    name: String,
    channel: u8,
}
pub type EffectInput = EffectIO;
pub type EffectOutput = EffectIO;

/// Implementation details of an Effect.
/// Either its route graph, a primitive effect, etc.
pub enum EffectData {
    RouteGraph(RouteGraph),
    Primitive(PrimitiveEffect),
}

/// Effects that cannot be decomposed; they have no implementation details and
/// must be implemented directly by the renderer.
#[derive(Debug, Copy, Clone)]
pub enum PrimitiveEffect {
    /// Primitive Delay effect
    Delay,
    /// Primitive Constant effect.
    /// Also serves as a unit step;
    /// Returns the float value for t >= 0, else 0.
    F32Constant,
    /// Primitive effect to multiply TWO input streams sample-wise.
    Multiply,
    /// Primitive effect to calculate A/B.
    /// Note: because not all floating point numbers have inverses,
    /// A * (1/B) != A/B. Hence, we need division (not inversion) for proper
    /// precision.
    Divide,
    /// Primitive effect to calculate A%B (true modulo; not remainder.
    /// Result, y,  is always positive: y is bounded by [0, B).
    Modulo,
    /// Primitive effect to return the sample-wise minimum of two input streams.
    /// Max(A, B) can be implemented as -Min(-A, -B).
    /// The choice to define Min instead of Max was mostly arbitrary,
    /// and chosen because Min is more common in linear programming to avoid dealing
    /// with Inf.
    Minimum,
}

impl Effect {
    pub fn are_slots_connected(&self, from_slot: u32, to_slot: u32) -> bool {
        match self.data {
            EffectData::RouteGraph(ref g) => g.are_slots_connected(from_slot, to_slot),
            // For primitive effects, we assume ALL slots are connected.
            _ => true,
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
        let prim_effect = id.get_primitive_url().map(|url| match url.path() {
            "/F32Constant" => PrimitiveEffect::F32Constant,
            "/Delay" => PrimitiveEffect::Delay,
            "/Multiply" => PrimitiveEffect::Multiply,
            "/Divide" => PrimitiveEffect::Divide,
            "/Modulo" => PrimitiveEffect::Modulo,
            "/Minimum" => PrimitiveEffect::Minimum,
            _ => panic!("Unrecognized primitive effect: {} (full url: {})", url.path(), url),
        });
        if let Some(prim_effect) = prim_effect {
            let me = Self {
                meta: EffectMeta {
                    name: id.name.clone(),
                    urls: id.urls.clone(),
                    // Primitive effects have undocumented I/O;
                    inputs: Default::default(),
                    outputs: Default::default(),
                },
                id: id,
                data: EffectData::Primitive(prim_effect),
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
                                data: EffectData::RouteGraph(graph),
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
    pub fn data(&self) -> &EffectData {
        &self.data
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
        // TODO: calculate sha using a smaller buffer
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
    pub fn new<U>(name: String, urls: U, inputs: HashMap<u32, EffectInput>, outputs: HashMap<u32, EffectOutput>) -> Self 
        where U: IntoIterator<Item=Url>
    {
        Self {
            name,
            urls: urls.into_iter().map(|url| url_serde::Serde(url)).collect(),
            inputs,
            outputs,
        }
    }
}

impl EffectIO {
    pub fn new(name: String, channel: u8) -> Self {
        Self{ name, channel }
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

