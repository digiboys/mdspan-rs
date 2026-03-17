use integer::Integer;

pub fn inner_product<T>(a: impl IntoIterator<Item = T>, b: impl IntoIterator<Item = T>) -> T
where
    T: Integer,
{
    std::iter::zip(a, b).map(|(a, b)| a * b).sum()
}
