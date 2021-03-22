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

// unsafe impl<T, const N: usize> UninitProject<[MaybeUninit<T>; N]> for [T; N] {
//     fn uninit_project(this: &mut MaybeUninit<Self>) -> &mut [MaybeUninit<T>; N] {
//         unsafe { &mut *this.as_mut_ptr().cast() }
//     }
// }

macro_rules! impl_UninitProject_for_array {
    ($($N:tt,)+) => {
        $(
            unsafe impl<T> UninitProject<[MaybeUninit<T>; $N]> for [T; $N] {
                fn uninit_project(this: &mut MaybeUninit<Self>) -> &mut [MaybeUninit<T>; $N] {
                    unsafe { &mut *this.as_mut_ptr().cast() }
                }
            }
        )+
    }
}

impl_UninitProject_for_array!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,);
impl_UninitProject_for_array!(17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32,);
impl_UninitProject_for_array!(64, 128, 256, 512, 1024, 2048, 4096,);

/// Sets the content of `T` to zero.
pub fn memset_zeroed<T>(this: &mut MaybeUninit<T>) {
    unsafe { this.as_mut_ptr().write_bytes(0, 1) }
}

/// Overwrites the content of `T`.
pub fn overwrite<T>(this: &mut MaybeUninit<T>, value: T) {
    unsafe { this.as_mut_ptr().write(value) }
}
