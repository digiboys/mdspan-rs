use integer::Integer;

pub trait Direction {
    fn build<const N: usize, T>(iter: impl DoubleEndedIterator<Item = T>) -> [T; N]
    where
        T: Integer;
}

pub struct Forward;
pub struct Reverse;

impl Direction for Forward {
    fn build<const N: usize, T>(iter: impl DoubleEndedIterator<Item = T>) -> [T; N]
    where
        T: Integer,
    {
        let mut acc = T::one();
        let mut iter = iter;
        core::array::from_fn(move |_| {
            let current = acc;
            acc *= iter.next().expect("fewer than N elements");
            current
        })
    }
}

impl Direction for Reverse {
    fn build<const N: usize, T>(iter: impl DoubleEndedIterator<Item = T>) -> [T; N]
    where
        T: Integer,
    {
        let mut acc = T::one();
        let mut iter = iter.rev();
        array_from_fn_rev::array_from_fn_rev(move |_| {
            let current = acc;
            acc *= iter.next().expect("fewer than N elements");
            current
        })
    }
}

pub fn exclusive_product_scan<D: Direction, const N: usize, T>(
    iter: impl IntoIterator<Item = T, IntoIter: DoubleEndedIterator>,
) -> [T; N]
where
    T: Integer,
{
    D::build(iter.into_iter())
}
