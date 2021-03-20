use crate::guard::with_guard;
use crate::SinglePlace;

use core::mem;

use rust_alloc::alloc::Layout;
use rust_alloc::alloc::{alloc, alloc_zeroed, dealloc, handle_alloc_error};
use rust_alloc::boxed::Box;

/// Allocates memory and initialize it.
pub fn emplace_by(layout: Layout, f: impl FnOnce(*mut ())) -> *mut () {
    unsafe { dyn_emplace(layout, alloc, f) }
}

/// Allocates zeroed memory and initialize it.
pub fn emplace_zeroed_by(layout: Layout, f: impl FnOnce(*mut ())) -> *mut () {
    unsafe { dyn_emplace(layout, alloc_zeroed, f) }
}

unsafe fn dyn_emplace(
    layout: Layout,
    a: unsafe fn(Layout) -> *mut u8,
    f: impl FnOnce(*mut ()),
) -> *mut () {
    let ptr = a(layout).cast::<()>();
    if ptr.is_null() {
        handle_alloc_error(layout)
    }
    with_guard(|| f(ptr), || dealloc(ptr.cast(), layout));
    ptr
}

impl<T> SinglePlace<T> for Box<T> {
    unsafe fn emplace_by(f: impl FnOnce(&mut mem::MaybeUninit<T>)) -> Self {
        let ptr = emplace_by(Layout::new::<T>(), |ptr| f(&mut *ptr.cast()));
        Box::from_raw(ptr.cast())
    }

    unsafe fn emplace_zeroed_by(f: impl FnOnce(&mut mem::MaybeUninit<T>)) -> Self {
        let ptr = emplace_zeroed_by(Layout::new::<T>(), |ptr| f(&mut *ptr.cast()));
        Box::from_raw(ptr.cast())
    }
}
