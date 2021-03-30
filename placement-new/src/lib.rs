//! Common utilities to implement "placement-new".

#![no_std]
#![deny(
    missing_docs,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]
#![allow(clippy::module_name_repetitions)]

#[cfg(feature = "alloc")]
extern crate alloc as rust_alloc;

#[cfg(feature = "alloc")]
mod alloc;

mod array;

mod guard;

mod place;

mod slice;

mod uninit;

#[doc(hidden)]
pub mod __private;

#[cfg(feature = "alloc")]
pub use self::alloc::*;

pub use self::array::*;

pub use self::place::*;

pub use self::slice::*;

pub use self::uninit::*;

#[cfg(feature = "derive")]
pub use placement_new_derive::UninitProject;
