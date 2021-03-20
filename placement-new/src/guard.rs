use core::mem::{self, ManuallyDrop};

pub struct Guard<F: FnOnce()>(ManuallyDrop<F>);

impl<F: FnOnce()> Drop for Guard<F> {
    fn drop(&mut self) {
        let f = unsafe { ManuallyDrop::take(&mut self.0) };
        f()
    }
}

impl<F: FnOnce()> Guard<F> {
    pub fn new(f: F) -> Self {
        Self(ManuallyDrop::new(f))
    }
    pub fn cancel(mut self) {
        unsafe { ManuallyDrop::drop(&mut self.0) };
        mem::forget(self)
    }
}

pub fn with_guard<R>(f: impl FnOnce() -> R, g: impl FnOnce()) -> R {
    let guard = Guard::new(g);
    let ret = f();
    guard.cancel();
    ret
}
