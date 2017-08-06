use std;
use std::collections::HashSet;
use std::io::Cursor;
use std::mem;
use std::ops::{Deref, Range};
use std::rc::Rc;

use digest::Digest;
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
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct EffectDesc {
    meta: EffectMeta,
    adjlist: AdjList,
}

/// Validated version of `EffectDesc`. Guaranteed to be synthesizable.
#[derive(Debug)]
pub struct Effect {
    meta: EffectMeta,
    // TODO: Effects are immutable, so we can make this an AdjList and store
    // the connectivity information separately.
    data: EffectData,
}

#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
pub struct EffectMeta {
    id: EffectId,
    inputs: Vec<EffectInput>,
    outputs: Vec<EffectOutput>,
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
#[derive(Debug)]
pub enum EffectData {
    RouteGraph(RouteGraph),
    Primitive(PrimitiveEffect),
}

/// Effects that cannot be decomposed; they have no implementation details and
/// must be implemented directly by the renderer.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PrimitiveEffect {
    /// Primitive Delay effect
    Delay,
    /// Primitive Constant effect.
    /// Also serves as a unit step;
    /// Returns the float value for t >= 0, else 0.
    F32Constant,
    /// Sum 2 floating point streams.
    Sum2,
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

/// Iterator over the outputs of a F32Constant primitive effect
pub struct F32ConstIterator {
    loc: Range<u32>,
}

impl Effect {
    pub fn are_slots_connected(&self, from_slot: u32, to_slot: u32) -> bool {
        match self.data {
            EffectData::RouteGraph(ref g) => g.are_slots_connected(from_slot, to_slot),
            // For primitive effects, we assume ALL slots are connected.
            _ => true,
        }
    }
    pub fn id(&self) -> &EffectId {
        &self.meta.id
    }
    pub fn meta(&self) -> &EffectMeta {
        &self.meta
    }
    /// Given the effect's information, and an interface by which to load
    /// resources, return an actual Effect.
    pub fn from_id(id: EffectId, resman: &ResMan) -> ResultE<Rc<Self>> {
        // For primitive effects, don't attempt to locate their descriptions (they don't exist)
        let prim_effect = id.get_primitive_url().and_then(PrimitiveEffect::from_url);
        // Attempt to instantiate a primitive effect, if the URL matched.
        if let Some(prim_effect) = prim_effect {
            if id.sha256 == None {
                let me = Self {
                    meta: EffectMeta {
                        // sha256 was already verified; no need to update it
                        id: id,
                        // Primitive effects have undocumented I/O;
                        inputs: Default::default(),
                        outputs: Default::default(),
                    },
                    data: EffectData::Primitive(prim_effect),
                };
                return Ok(Rc::new(me));
            } else {
                warn!("Attempted to load a primitive Effect, but cannot because of mismatched sha256: {:?}", id.sha256);
            }
        }

        // Locate descriptions for non-primitive effects
        for (path, reader) in resman.find_effect(&id) {
            // Try to deserialize to an effect description
            let desc: Result<EffectDesc, serde_json::Error> = serde_json::from_reader(reader);
            match desc {
                Ok(mut desc) => {
                    if desc.meta.id.name() == id.name() {
                        desc.update_id();
                        match RouteGraph::from_adjlist(desc.adjlist, resman) {
                            Ok(graph) => {
                                let me = Self {
                                    meta: desc.meta,
                                    data: EffectData::RouteGraph(graph),
                                };
                                // TODO: implement some form of caching
                                return Ok(Rc::new(me));
                            },
                            Err(error) => warn!("[{:?}] RouteGraph::from_adjlist failed: {:?}", path, error)
                        }
                    } else {
                        trace!("[{:?}] Effect names differ: wanted {:?} got {:?}", path, id.name(), desc.meta.id.name());
                    }
                },
                Err(error) => {
                    warn!("[{:?}] Unable to deserialize EffectDesc: {:?}", path, error)
                }
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
            urls: urls.into_iter().map(url_serde::Serde).collect(),
        }
    }
    pub fn name(&self) -> &str {
        &self.name
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
    pub fn meta(&self) -> &EffectMeta {
        &self.meta
    }
    /// Make sure the id is fully populated with hashes, etc.
    fn update_id(&mut self) {
        if self.meta.id.sha256.is_none() {
            // TODO: calculate sha using a smaller buffer
            let as_vec = serde_json::to_vec(self).unwrap();
            let result = Sha256::digest_reader(&mut Cursor::new(as_vec)).unwrap();
            let mut hash: [u8; 32] = Default::default();
            hash.copy_from_slice(result.as_slice());
            self.meta.id.sha256 = Some(hash);
        }
    }
}

impl EffectMeta {
    pub fn new<U>(name: String, urls: U, inputs: Vec<EffectInput>, outputs: Vec<EffectOutput>) -> Self 
        where U: IntoIterator<Item=Url>
    {
        Self {
            id: EffectId::new(name, None, urls),
            inputs,
            outputs,
        }
    }
    pub fn name(&self) -> &str {
        self.id.name()
    }
    fn inputs<'a>(&'a self) -> Box<Iterator<Item=EffectInput> + 'a> {
        match self.prim_effect() {
            Some(PrimitiveEffect::Delay) => Box::new(vec![
                    EffectInput::new("source".into(), 0),
                    EffectInput::new("frames".into(), 0)
                ].into_iter()),
            Some(PrimitiveEffect::F32Constant) => Box::new([].iter().cloned()),
            Some(PrimitiveEffect::Sum2) | Some(PrimitiveEffect::Multiply) | Some(PrimitiveEffect::Minimum) => Box::new(vec![
                    EffectInput::new("source".into(), 0),
                    EffectInput::new("source2".into(), 0),
                ].into_iter()),
            Some(PrimitiveEffect::Divide) | Some(PrimitiveEffect::Modulo) => Box::new(vec![
                    EffectInput::new("source".into(), 0),
                    EffectInput::new("divisor".into(), 0),
                ].into_iter()),
            _ => Box::new(self.inputs.iter().cloned())
        }
    }
    fn outputs<'a>(&'a self) -> Box<Iterator<Item=EffectOutput> + 'a> {
        match self.prim_effect() {
            Some(PrimitiveEffect::F32Constant) => Box::new(F32ConstIterator::new()),
            Some(_) => Box::new(Some(EffectOutput::new("result".into(), 0)).into_iter()),
            None => Box::new(self.outputs.iter().cloned())
        }
    }
    pub fn inputs_by_name<'a>(&'a self, name: &'a str) -> impl Iterator<Item=EffectInput> + 'a {
        self.inputs().filter(move |item| item.name() == name)
    }
    pub fn outputs_by_name<'a>(&'a self, name: &'a str) -> impl Iterator<Item=EffectOutput> + 'a {
        self.outputs().filter(move |item| item.name() == name)
    }
    pub fn is_valid_input(&self, slotno: u32) -> bool {
        self.inputs().nth(slotno as usize).is_some()
    }
    pub fn is_valid_output(&self, slotno: u32) -> bool {
        self.outputs().nth(slotno as usize).is_some()
    }
    fn prim_effect(&self) -> Option<PrimitiveEffect> {
        self.id.get_primitive_url().and_then(PrimitiveEffect::from_url)
    }
}

impl EffectIO {
    pub fn new(name: String, channel: u8) -> Self {
        Self{ name, channel }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn channel(&self) -> u8 {
        self.channel
    }
    pub fn is_source(&self) -> bool {
        self.name == "source"
    }
    pub fn is_result(&self) -> bool {
        self.name == "result"
    }
}

impl PrimitiveEffect {
    fn from_url(url: &Url) -> Option<Self> {
        if url.scheme() == "primitive" {
            match url.path() {
                "/Delay"       => Some(PrimitiveEffect::Delay),
                "/F32Constant" => Some(PrimitiveEffect::F32Constant),
                "/Sum2"        => Some(PrimitiveEffect::Sum2),
                "/Multiply"    => Some(PrimitiveEffect::Multiply),
                "/Divide"      => Some(PrimitiveEffect::Divide),
                "/Modulo"      => Some(PrimitiveEffect::Modulo),
                "/Minimum"     => Some(PrimitiveEffect::Minimum),
                _ => {
                    warn!("Unrecognized primitive effect: {} (full url: {})", url.path(), url);
                    None
                }
            }
        } else {
            None
        }
    }
}

impl PartialEq for EffectMeta {
    // Equality implemented in a way where we can easily check things like
    //   "Is this the same primitive Delay effect this renderer knows how to implement?"
    fn eq(&self, other: &EffectMeta) -> bool {
        self.id.name == other.id.name &&
            self.id.urls == other.id.urls
    }
}
impl Eq for EffectMeta {}


impl F32ConstIterator {
    fn new() -> Self {
        Self{ loc: (0..std::u32::MAX) }
    }
    fn to_f32(v: u32) -> f32 {
        unsafe { mem::transmute(v) }
    }
    fn to_output(v: u32) -> EffectOutput {
        let name = format!("const{}", Self::to_f32(v));
        EffectOutput::new(name, 0)
    }
}

impl Iterator for F32ConstIterator {
    type Item = EffectOutput;
    fn next(&mut self) -> Option<Self::Item> {
        self.loc.next().map(Self::to_output)
    }
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.loc.nth(n).map(Self::to_output)
    }
}
