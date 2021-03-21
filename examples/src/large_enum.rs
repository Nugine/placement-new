use std::mem::MaybeUninit;
use std::ptr;

use placement_new::{memset_zeroed, overwrite, uninit_project, SinglePlace, UninitProject};

#[allow(clippy::large_enum_variant)]
#[derive(UninitProject)]
#[repr(C)]
pub enum State {
    A,
    B(String),
    C(u32, u32),
    D { name: String, value: String },
    E { data: [u8; 4096] },
}

impl State {
    pub fn init_a(this: &mut MaybeUninit<Self>) {
        uninit_project!(this => enum State => A);
    }

    pub fn init_d(this: &mut MaybeUninit<Self>, name: String, value: String) {
        let this = uninit_project!(this => enum State => D);
        overwrite(&mut this.name, name);
        overwrite(&mut this.value, value);
    }

    pub fn init_e(this: &mut MaybeUninit<Self>) {
        let this = uninit_project!(this => enum State => E);
        memset_zeroed(&mut this.data);
    }

    pub fn reset_to_a(&mut self) {
        let this: *mut Self = self;
        unsafe {
            ptr::drop_in_place(this);
            Self::init_a(&mut *this.cast())
        }
    }

    pub fn new_boxed_e() -> Box<Self> {
        unsafe { Box::emplace_zeroed_by(Self::init_e) }
    }
}

#[test]
fn check_boxed() {
    let mut s = State::new_boxed_e();
    match *s {
        State::E { ref data } => assert!(data.iter().fold(0, |acc, x| acc | x) == 0),
        _ => panic!(),
    };

    s.reset_to_a();
    match *s {
        State::A => {}
        _ => panic!(),
    }

    drop(s);
}
