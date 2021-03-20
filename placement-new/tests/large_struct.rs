#![deny(clippy::all)]

use std::collections::HashMap;
use std::mem::MaybeUninit;

use placement_new::{memset_zeroed, overwrite, uninit_project, SinglePlace, UninitProject};

#[derive(UninitProject)]
#[repr(C)]
pub struct State {
    a: u64,
    b: String,
    c: HashMap<String, String>,
    d: [Vec<u32>; 16],
    e: [u8; 4096],
}

impl State {
    /// Initializes a [`State`] completely.
    pub fn init(this: &mut MaybeUninit<Self>) {
        Self::init_zeroed(this);

        let this = uninit_project(this);
        overwrite(&mut this.a, 0);
        memset_zeroed(&mut this.e)
    }

    /// Initializes a [`State`] partially.
    /// Assumes that `this` is filled with zero.
    pub fn init_zeroed(this: &mut MaybeUninit<Self>) {
        let this = uninit_project(this);

        overwrite(&mut this.b, String::new());
        overwrite(&mut this.c, HashMap::new());

        for v in uninit_project(&mut this.d) {
            overwrite(v, Vec::new())
        }
    }

    pub fn new_boxed() -> Box<Self> {
        // # Safety
        // The allocated memory is filled with zero.
        // So `init_zeroed` can initializes `Self` correctly.
        unsafe { Box::emplace_zeroed_by(Self::init_zeroed) }
    }
}

#[test]
fn check_boxed() {
    let s = State::new_boxed();

    assert!(s.a == 0);
    assert!(s.b.is_empty());
    assert!(s.c.is_empty());
    assert!(s.d.iter().all(|v| v.is_empty()));
    assert!(s.e.iter().fold(0, |acc, x| acc | x) == 0);

    drop(s);
}
