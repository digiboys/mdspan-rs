pub trait Constant {
    type ArrayOf<T: Copy>: IntoIterator<Item = T> + Copy + std::ops::IndexMut<usize, Output = T> + AsMut<[T]>;
    const VALUE: usize;
}

pub struct Const<const N: usize>;

impl<const N: usize> Constant for Const<N> {
    type ArrayOf<T: Copy> = [T; N];
    const VALUE: usize = N;
}
