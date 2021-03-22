use crate::guard::guard_on_unwind;

use core::mem::{self, MaybeUninit};
use core::ptr;

/// Initializes a slice.
///
/// `f` takes two arguments: the element's index and pointer.
///
/// If `f` panics, every `T` initialized by `f` will be dropped.
///
/// # Safety
/// + `f` must initialize `T` correctly every time.
#[inline]
pub unsafe fn init_slice_with<T>(
    slice: &mut [MaybeUninit<T>],
    mut f: impl FnMut(usize, &mut MaybeUninit<T>),
) -> &mut [T] {
    let mut count: usize = 0;

    let count: *mut usize = &mut count;
    let slice: *mut [MaybeUninit<T>] = slice;

    guard_on_unwind(
        || {
            for this in &mut *slice {
                let idx = *count;
                f(idx, this);
                *count += 1;
            }
        },
        || {
            if mem::needs_drop::<T>() {
                let partial_slice = (*slice).get_unchecked_mut(..*count);
                for this in partial_slice {
                    ptr::drop_in_place(this.as_mut_ptr());
                }
            }
        },
    );

    #[allow(clippy::transmute_ptr_to_ref)]
    {
        mem::transmute(slice)
    }
}
