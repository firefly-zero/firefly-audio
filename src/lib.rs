#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![deny(
    rust_2018_idioms,
    redundant_lifetimes,
    redundant_semicolons,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::allow_attributes
)]
#![allow(clippy::wildcard_imports)]
#![expect(
    // TODO: fix casting warning
    clippy::cast_precision_loss,
    clippy::module_name_repetitions,
    clippy::new_without_default,
)]
extern crate alloc;

mod basic_types;
mod error;
mod manager;
pub mod modulators;
mod node;
mod pcm;
mod processor;
mod processors;
mod sources;

pub use basic_types::*;
pub use error::*;
pub use manager::*;
pub use node::*;
pub use pcm::*;
pub use processor::*;
pub use processors::*;
pub use sources::*;
