#![cfg(test)]
use integer_utils::IntegerUtils;

#[test]
fn test_round_down_to_multiple_simple_cases() {
    assert_eq!(9, usize::round_down_to_multiple(11, 3));
    assert_eq!(21, usize::round_down_to_multiple(21, 7));
    assert_eq!(100, usize::round_down_to_multiple(100, 10));
    assert_eq!(16, usize::round_down_to_multiple(17, 16));
}

#[test]
fn test_round_up_to_multiple_simple_cases() {
    assert_eq!(12, usize::round_up_to_multiple(11, 3));
    assert_eq!(21, usize::round_up_to_multiple(21, 7));
    assert_eq!(100, usize::round_up_to_multiple(100, 10));
    assert_eq!(32, usize::round_up_to_multiple(17, 16));
}

quickcheck! {
    fn qc_round_down_to_multiple(val         : usize,
                                 multiple_of : usize) -> bool {
        let multiple_of = if multiple_of == 0 { 1 } else { multiple_of };

        let res = usize::round_down_to_multiple(val, multiple_of);

        res <= val
            && ((val > 0 && ((val % multiple_of == 0 && res == val)
                             || (val % multiple_of != 0 && res <  val)))
                || val == 0)
            && usize::round_down_to_multiple(val, multiple_of) % multiple_of == 0
    }
}
