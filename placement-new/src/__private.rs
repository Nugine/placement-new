#![allow(missing_docs, clippy::missing_safety_doc)]

pub use core;

#[allow(clippy::missing_const_for_fn)] // FIXME: false positive
#[inline]
pub unsafe fn split_enum<T, P>(base: *mut ()) -> (*mut T, *mut P) {
    let base = base.cast::<u8>();
    let tag = base.cast::<T>();
    let payload = base.add(core::mem::size_of::<T>()).cast::<P>();
    (tag, payload)
}

#[cfg(feature = "derive")]
pub use placement_new_derive::__uninit_project_variant;
