// needed for returning Iterator traits
#![feature(conservative_impl_trait)]

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate sha2;
extern crate url;
extern crate url_serde;

#[macro_use]
extern crate osc_address_derive;
extern crate osc_address;
extern crate serde_osc;

pub mod dispatch;
pub mod render;
pub mod routing;
pub mod resman;

