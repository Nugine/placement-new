use core::mem::MaybeUninit;

/// A type with determined layout.
/// It can be projected to another type with the same fields,
/// but all the fields are not initialized yet.
pub unsafe trait UninitProject<U>: Sized {
    /// Projects a type to its uninitialized mirror.
    fn uninit_project(this: &mut MaybeUninit<Self>) -> &mut U;
}

/// Projects a type to its uninitialized mirror.
#[cfg(feature = "derive")]
#[macro_export]
macro_rules! uninit_project {
    ($this:expr) => {{
        $crate::UninitProject::<_>::uninit_project($this)
    }};
    ($this:expr => enum $ty:path => $variant:ident) => {{
        $crate::__private::__uninit_project_variant!($ty => $variant)($this)
    }};
}

unsafe impl<T, const N: usize> UninitProject<[MaybeUninit<T>; N]> for [T; N] {
    fn uninit_project(this: &mut MaybeUninit<Self>) -> &mut [MaybeUninit<T>; N] {
        unsafe { &mut *this.as_mut_ptr().cast() }
    }
}

/// Sets the content of `T` to zero.
#[inline]
pub fn memset_zeroed<T>(this: &mut MaybeUninit<T>) {
    unsafe { this.as_mut_ptr().write_bytes(0, 1) }
}

/// Overwrites the content of `T`.
#[inline]
pub fn overwrite<T>(this: &mut MaybeUninit<T>, value: T) {
    unsafe { this.as_mut_ptr().write(value) }
}
