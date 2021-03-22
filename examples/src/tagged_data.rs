use std::alloc::{dealloc, Layout};
use std::slice;
use std::{fmt, mem, ptr};

pub struct Data(*mut ());

unsafe impl Send for Data {}
unsafe impl Sync for Data {}

struct Header {
    name: &'static str,
    len: usize,
}

impl Data {
    pub fn new_zeroed(name: &'static str, len: usize) -> Self {
        let size = mem::size_of::<Header>()
            .checked_add(len)
            .expect("size overflow");

        let align = mem::align_of::<Header>();
        let layout = Layout::from_size_align(size, align).expect("invalid layout");

        let ptr = placement_new::emplace_zeroed_with(layout, |ptr| unsafe {
            ptr.cast::<Header>().write(Header { name, len })
        });

        Self(ptr.cast())
    }

    fn header(&self) -> &Header {
        unsafe { &*self.0.cast() }
    }

    fn raw_mut_body(&self) -> *mut u8 {
        unsafe { self.0.cast::<u8>().add(mem::size_of::<Header>()) }
    }

    pub fn name(&self) -> &str {
        self.header().name
    }

    pub fn len(&self) -> usize {
        self.header().len
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn as_bytes(&self) -> &[u8] {
        let len = self.len();
        let ptr = self.raw_mut_body();
        unsafe { slice::from_raw_parts(ptr, len) }
    }

    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        let len = self.len();
        let ptr = self.raw_mut_body();
        unsafe { slice::from_raw_parts_mut(ptr, len) }
    }
}

impl Drop for Data {
    fn drop(&mut self) {
        let len = self.len();
        unsafe {
            ptr::drop_in_place(self.0.cast::<Header>());

            let size = mem::size_of::<Header>().wrapping_add(len);
            let align = mem::align_of::<Header>();
            let layout = Layout::from_size_align_unchecked(size, align);
            dealloc(self.0.cast(), layout)
        }
    }
}

impl fmt::Debug for Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Data")
            .field("name", &self.name())
            .field("len", &self.len())
            .field("bytes", &self.as_bytes())
            .finish()
    }
}

#[test]
fn check() {
    let name = "hello";
    let len = 4096;
    let data = Data::new_zeroed(name, len);

    assert_eq!(data.name(), name);
    assert_eq!(data.len(), len);
    assert!(data.as_bytes().iter().fold(0, |acc, x| acc | x) == 0);

    drop(data);
}
