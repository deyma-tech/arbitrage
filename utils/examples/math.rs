use num_traits::ToPrimitive;

fn main() {
    let a = 123.0;
    assert_eq!(a.to_u64(), Some(123));

    let b = 1e20; // mimo rozsah u64
    assert_eq!(b.to_u64(), None);

    let c = 3.14;
    assert_eq!(c.to_u64(), Some(3));

    let d = -3.14;
    assert_eq!(d.to_u64(), None);
}
