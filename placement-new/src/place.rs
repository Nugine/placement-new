use core::mem::MaybeUninit;

/// A place containing a single value.
pub trait SinglePlace<T> {
    /// # Safety
    /// `f` must initialize `T` correctly.
    unsafe fn emplace_with(f: impl FnOnce(&mut MaybeUninit<T>)) -> Self;

    /// # Safety
    /// `f` must initialize `T` correctly.
    unsafe fn emplace_zeroed_with(f: impl FnOnce(&mut MaybeUninit<T>)) -> Self;
}

/// A place containing multiple continuous values.
pub trait SlicePlace<T> {
    /// # Safety
    /// `f` must initialize every `T` correctly.
    unsafe fn emplace_with(len: usize, f: impl FnOnce(&mut [MaybeUninit<T>])) -> Self;

    /// # Safety
    /// `f` must initialize every `T` correctly.
    unsafe fn emplace_zeroed_with(len: usize, f: impl FnOnce(&mut [MaybeUninit<T>])) -> Self;
}
