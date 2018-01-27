pub trait IntegerUtils {
    type Item;
    fn use_then_add1(&mut Self::Item) -> Self::Item;
}

macro_rules! impl_integer_utils {
    (
        $( $x:ty ),+
    ) => {
        $(
            impl IntegerUtils for $x {
                type Item = $x;

                fn use_then_add1(val : &mut Self::Item) -> Self::Item {
                    let tmp = *val;
                    *val += 1;
                    tmp
                }
            }
        )+
    }
}

impl_integer_utils!(u8, u16, u32, u64, usize,
                    i8, i16, i32, i64, isize);
