/// Compares two Vectors
pub(crate) fn cmp_vec<T: std::cmp::PartialEq>(vec: &[T], vec2: &[T]) -> bool {
    if vec.len() != vec2.len() {
        return false;
    }

    for i in 0..vec.len() {
        if vec[i] != vec2[i] {
            return false;
        }
    }
    true
}
