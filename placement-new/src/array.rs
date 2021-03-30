use crate::UninitProject;

use core::mem::MaybeUninit;

/// Creates an array.
///
/// `f` take one argument: the element's index.
///
/// If `f` panics, every `T` initialized by `f` will be dropped.
pub fn create_array_with<T, const N: usize>(mut f: impl FnMut(usize) -> T) -> [T; N] {
    let mut array: MaybeUninit<[T; N]> = MaybeUninit::uninit();

    unsafe {
        crate::init_slice_with(UninitProject::uninit_project(&mut array), |idx, this| {
            crate::overwrite(this, f(idx))
        });
        array.assume_init()
    }
}
