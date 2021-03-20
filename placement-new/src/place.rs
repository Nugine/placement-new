use core::mem::MaybeUninit;

/// A place containing a single value.
pub trait SinglePlace<T> {
    /// # Safety
    /// `f` must initialize `T` correctly.
    unsafe fn emplace_by(f: impl FnOnce(&mut MaybeUninit<T>)) -> Self;

    /// # Safety
    /// `f` must initialize `T` correctly.
    unsafe fn emplace_zeroed_by(f: impl FnOnce(&mut MaybeUninit<T>)) -> Self;
}
