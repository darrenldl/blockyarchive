pub trait IntegerUtils {
    type Item;
    fn round_down_to_multiple(_: Self::Item, _: Self::Item) -> Self::Item;
    fn round_up_to_multiple(_: Self::Item, _: Self::Item) -> Self::Item;
    fn ensure_at_least(_: Self::Item, _: Self::Item) -> Self::Item;
    fn ensure_at_most(_: Self::Item, _: Self::Item) -> Self::Item;
}

macro_rules! impl_integer_utils {
    (
        $( $x:ty ),+
    ) => {
        $(
            impl IntegerUtils for $x {
                type Item = $x;

                fn round_down_to_multiple(val         : Self::Item,
                                          multiple_of : Self::Item)
                                          -> Self::Item {
                    (val / multiple_of) * multiple_of
                }

                fn round_up_to_multiple(val         : Self::Item,
                                        multiple_of : Self::Item)
                                        -> Self::Item {
                    ((val + (multiple_of - 1)) / multiple_of) * multiple_of
                }

                fn ensure_at_least(val      : Self::Item,
                                   at_least : Self::Item)
                                   -> Self::Item {
                    use std::cmp::max;
                    max(val, at_least)
                }

                fn ensure_at_most(val     : Self::Item,
                                  at_most : Self::Item)
                                  -> Self::Item {
                    use std::cmp::min;
                    min(val, at_most)
                }
            }
        )+
    }
}

impl_integer_utils!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);
