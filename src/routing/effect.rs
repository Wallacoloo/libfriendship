// TODO: rust-crypto lib for SHA256 verification.

extern crate serde;
extern crate serde_json;
use self::serde_json::map::Map;
use self::serde_json::value::Value;

#[derive(Serialize, Deserialize)]
struct Effect {
    /// Canonical name of the effect
    name: String,
    /// Hash of the effect's definition file, or None if the effect is primitive
    sha1: Option<[u8; 32]>,
    /// List of URLs where the Effect can be obtained
    urls: Vec<String>,
    /// Arguments specific to the given effect (relevant for e.g. Constant).
    effect_args: Map<String, Value>,
}


///// Data structure needed for deserializing Effects
//struct EffectVisitor {
//}
//impl EffectVisitor {
//    fn new() -> Self {
//        EffectVisitor {}
//    }
//}
//impl de::Visitor for EffectVisitor {
//    // This Visitor deserializes to an Effect.
//    type Value = Effect;
//    // Diagnostic info
//    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//        formatter.write_str("Effect")
//    }
//    fn visit_map<M>(self, mut visitor: M) -> Result<Self::Value, M::Error>
//        where M: de::MapVisitor
//    {
//        // First, deserialize into a generic Map<String, Value>
//        let mut collected = Map::new();
//
//        // While there are entries remaining in the input, add them
//        // into our map.
//        while let Some((key, value)) = visitor.visit()? {
//            collected.insert(key, value);
//        }
//
//        // typecheck all keys that we understand.
//        let name = match collected.entry("name") {
//            Vacant(_) => M::Error::missing_field("name"),
//            Occupied(entry) => 
//        }
//
//        Ok(values)
//    }
//}
