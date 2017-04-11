// needed for returning Iterator traits
#![feature(conservative_impl_trait)]

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate sha2;
extern crate url;
extern crate url_serde;

pub mod render;
pub mod routing;
pub mod resman;

