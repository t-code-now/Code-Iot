//! Small deterministic shuffle (xorshift64* + Fisher–Yates) so the core
//! needs no `rand` dependency and stays testable.

fn next(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    *state = x;
    x.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

/// Shuffle `items` with `seed`. When there are at least two distinct items
/// the result is guaranteed to differ from the input order, so a reorder
/// exercise never starts already solved.
pub fn shuffled<T: Clone + PartialEq>(items: &[T], seed: u64) -> Vec<T> {
    let mut out = items.to_vec();
    let mut state = seed | 1; // xorshift state must be non-zero
    for i in (1..out.len()).rev() {
        let j = (next(&mut state) % (i as u64 + 1)) as usize;
        out.swap(i, j);
    }
    if out.as_slice() == items && items.iter().any(|x| *x != items[0]) {
        out.rotate_left(1);
        if out.as_slice() == items {
            out.reverse();
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_elements_and_changes_order() {
        let items = vec![1, 2, 3, 4, 5];
        for seed in 0..500 {
            let s = shuffled(&items, seed);
            assert_ne!(s, items);
            let mut sorted = s.clone();
            sorted.sort();
            assert_eq!(sorted, items);
        }
        assert_eq!(shuffled(&[7], 3), vec![7]);
        assert_eq!(shuffled(&[1, 1], 3), vec![1, 1]);
        let empty: Vec<u8> = vec![];
        assert!(shuffled(&empty, 1).is_empty());
    }
}
