// needed for returning Iterator traits
#![feature(conservative_impl_trait)]
// needed for some Option methods (note: on track for stabilization)
//#![feature(option_entry)]

extern crate digest;
extern crate jagged_array;
extern crate llvm;
extern crate llvm_sys;
#[macro_use] extern crate log;
extern crate ndarray;
extern crate num;
#[macro_use] extern crate osc_address_derive;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate sha2;
extern crate streaming_iterator;
extern crate url;
extern crate url_serde;


pub mod client;
pub mod dispatch;
pub mod render;
pub mod routing;
pub mod resman;


pub use dispatch::Dispatch;
pub use client::Client;
