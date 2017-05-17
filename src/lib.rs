// needed for returning Iterator traits
#![feature(conservative_impl_trait)]

extern crate byteorder;
#[macro_use] extern crate defmac;
extern crate digest;
extern crate filebuffer;
extern crate num;
#[macro_use] extern crate osc_address_derive;
extern crate osc_address;
extern crate serde;
extern crate serde_osc;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate sha2;
extern crate url;
extern crate url_serde;


pub mod client;
pub mod dispatch;
pub mod render;
pub mod routing;
pub mod resman;
#[macro_use] pub mod util;
pub mod stdfx;


pub use dispatch::Dispatch;
pub use client::Client;
