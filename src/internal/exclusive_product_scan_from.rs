use integer::Integer;

pub fn exclusive_product_scan_from<T: Integer>(iter: impl Iterator<Item = T>) -> impl FnMut() -> T {
    let mut acc = T::one();
    let mut iter = iter;
    move || {
        let current = acc;
        acc *= iter.next().expect("input iterator exhausted");
        current
    }
}
