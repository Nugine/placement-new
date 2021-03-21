//! Common utilities to implement "placement-new".

#![no_std]
#![deny(missing_docs, clippy::all, clippy::cargo)]

#[cfg(feature = "alloc")]
extern crate alloc as rust_alloc;

#[cfg(feature = "alloc")]
mod alloc;

mod guard;

mod place;

mod uninit;

#[doc(hidden)]
pub mod __private;

#[cfg(feature = "alloc")]
pub use self::alloc::*;

pub use self::place::*;

pub use self::uninit::*;

#[cfg(feature = "derive")]
pub use placement_new_derive::UninitProject;
