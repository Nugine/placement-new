use placement_new::{overwrite, SlicePlace};

#[must_use]
pub fn zeroed_bytes(len: usize) -> Box<[u8]> {
    unsafe { Box::emplace_zeroed_with(len, |_| {}) }
}

pub fn repeat<T: Clone>(value: &T, n: usize) -> Box<[T]> {
    unsafe {
        Box::emplace_zeroed_with(n, |slice| {
            placement_new::init_slice_with(slice, |_, this| overwrite(this, value.clone()));
        })
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::panic;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn check_zeroed_bytes() {
        let b = super::zeroed_bytes(4096);
        assert!(b.iter().fold(0, |acc, x| acc | x) == 0);
        drop(b);
    }

    #[test]
    fn check_repeat() {
        let ss = super::repeat(&String::from("hello"), 128);
        assert!(ss.iter().all(|s| s == "hello"));
        drop(ss)
    }

    #[test]
    fn check_repeat_unwind() {
        struct Foo {
            payload: String,
            ttl: Cell<usize>,
        }

        impl Clone for Foo {
            fn clone(&self) -> Self {
                let ttl = self.ttl.get();
                if ttl == 0 {
                    panic!()
                }
                self.ttl.set(ttl - 1);
                Self {
                    payload: self.payload.clone(),
                    ttl: self.ttl.clone(),
                }
            }
        }

        static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

        impl Drop for Foo {
            fn drop(&mut self) {
                DROP_COUNT.fetch_add(1, Ordering::Relaxed);
            }
        }

        let origin = Foo {
            payload: String::from("hello"),
            ttl: Cell::new(64),
        };

        panic::catch_unwind(panic::AssertUnwindSafe(|| super::repeat(&origin, 128))).ok();

        assert_eq!(DROP_COUNT.load(Ordering::Relaxed), 64);
    }
}
