#[inline]
#[must_use]
pub fn index_array() -> [usize; 16] {
    placement_new::create_array_with(|idx| idx)
}

#[inline]
#[must_use]
pub fn vec_array() -> [Vec<u8>; 16] {
    placement_new::create_array_with(|_| Vec::new())
}

#[test]
fn check_index_array() {
    let array = index_array();
    array
        .iter()
        .enumerate()
        .for_each(|(idx, &x)| assert_eq!(idx, x))
}

#[test]
fn check_vec_array() {
    let array = vec_array();
    array.iter().for_each(|v| assert!(v.is_empty()))
}
