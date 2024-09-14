#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]
#![allow(clippy::wildcard_imports)]
// TODO: fix casting warnings
#![expect(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_lossless,
    clippy::module_name_repetitions,
    clippy::new_without_default
)]
extern crate alloc;

mod basic_types;
mod error;
mod manager;
pub mod modulators;
mod node;
mod processor;
mod processors;
mod sources;

pub use basic_types::*;
pub use error::*;
pub use manager::*;
pub use node::*;
pub use processor::*;
pub use processors::*;
pub use sources::*;
