use core::mem::{self, ManuallyDrop};

pub struct Guard<F: FnOnce()>(ManuallyDrop<F>);

impl<F: FnOnce()> Drop for Guard<F> {
    fn drop(&mut self) {
        let f = unsafe { ManuallyDrop::take(&mut self.0) };
        f()
    }
}

impl<F: FnOnce()> Guard<F> {
    #[inline]
    pub fn new(f: F) -> Self {
        Self(ManuallyDrop::new(f))
    }

    #[inline]
    pub fn cancel(mut self) {
        unsafe { ManuallyDrop::drop(&mut self.0) };
        mem::forget(self)
    }
}

#[inline]
pub fn guard_on_unwind<R>(f: impl FnOnce() -> R, g: impl FnOnce()) -> R {
    let guard = Guard::new(g);
    let ret = f();
    guard.cancel();
    ret
}
