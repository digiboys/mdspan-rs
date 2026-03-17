// pub use mod_accessor::*;
// pub use mod_layout::*;
// pub use mod_mdspan::*;

use core::{
    iter::Sum,
    ops::{Add, Deref, Mul},
};
use std::marker::PhantomData;

pub trait Layout {
    type IndexTypeN;
    type IndexType1;

    fn flatten<TIndices>(&self, indices: &TIndices) -> Option<Self::IndexType1>
    where
        for<'a> &'a TIndices: IntoIterator,
        for<'a> <&'a TIndices as IntoIterator>::Item: Deref,
        for<'a> <<&'a TIndices as IntoIterator>::Item as Deref>::Target: Copy,
        for<'a> Self::IndexTypeN: From<<<&'a TIndices as IntoIterator>::Item as Deref>::Target>;
}

pub struct AxialLayout<TExtents, TStrides, IndexTypeN, IndexType1> {
    extents: TExtents,
    strides: TStrides,
    index_type_n: PhantomData<IndexTypeN>,
    index_type_1: PhantomData<IndexType1>,
}

trait CopyFromDeref<T> {
    fn copy_from_deref(value: T) -> Self;
}

impl<T, U> CopyFromDeref<U> for T
where
    U: Deref,
    <U as Deref>::Target: Copy,
    T: From<<U as Deref>::Target>,
{
    #[inline]
    fn copy_from_deref(value: U) -> Self {
        From::from(*value)
    }
}

impl<TExtents, TStrides, IndexTypeN, IndexType1> Layout for AxialLayout<TExtents, TStrides, IndexTypeN, IndexType1>
where
    for<'a> &'a TExtents: IntoIterator,
    for<'a> <&'a TExtents as IntoIterator>::Item: Deref,
    for<'a> <<&'a TExtents as IntoIterator>::Item as Deref>::Target: Copy,
    for<'a> IndexTypeN: From<<<&'a TExtents as IntoIterator>::Item as Deref>::Target>,
    for<'a> &'a TStrides: IntoIterator,
    for<'a> <&'a TStrides as IntoIterator>::Item: Deref,
    for<'a> <<&'a TStrides as IntoIterator>::Item as Deref>::Target: Copy,
    for<'a> IndexType1: From<<<&'a TStrides as IntoIterator>::Item as Deref>::Target>,
    IndexType1: From<IndexTypeN> + Mul<Output = IndexType1> + Add<Output = IndexType1> + Sum,
    IndexTypeN: Ord,
{
    type IndexTypeN = IndexTypeN;
    type IndexType1 = IndexType1;

    fn flatten<TIndices>(&self, indices: &TIndices) -> Option<Self::IndexType1>
    where
        for<'a> &'a TIndices: IntoIterator,
        for<'a> <&'a TIndices as IntoIterator>::Item: Deref,
        for<'a> <<&'a TIndices as IntoIterator>::Item as Deref>::Target: Copy,
        for<'a> IndexTypeN: From<<<&'a TIndices as IntoIterator>::Item as Deref>::Target>,
    {
        if std::iter::zip(indices, &self.extents).all(|(i, e)| IndexTypeN::from(*i) < IndexTypeN::from(*e)) {
            Some(
                std::iter::zip(indices, &self.strides)
                    .map(|(i, s)| IndexType1::from(IndexTypeN::from(*i)) * IndexType1::from(*s))
                    .sum(),
            )
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ChunkExtents;

    impl<'a> IntoIterator for &'a ChunkExtents {
        type Item = &'a u8;

        type IntoIter = std::slice::Iter<'a, u8>;

        fn into_iter(self) -> Self::IntoIter {
            [32; 3].iter()
        }
    }

    struct ChunkStrides;

    impl<'a> IntoIterator for &'a ChunkStrides {
        type Item = &'a u16;
        type IntoIter = std::slice::Iter<'a, u16>;

        fn into_iter(self) -> Self::IntoIter {
            [32 * 32, 32, 1].iter()
        }
    }

    trait TupleIntoArray<T, const N: usize> {
        fn into_array(self) -> [T; N];
    }

    impl<T0, T1, T> TupleIntoArray<T, 2> for (T0, T1)
    where
        T0: Into<T>,
        T1: Into<T>,
    {
        fn into_array(self) -> [T; 2] {
            [self.0.into(), self.1.into()]
        }
    }

    #[test]
    fn x() {
        let layout = AxialLayout {
            extents: ChunkExtents,
            strides: ChunkStrides,
            index_type_n: PhantomData::<u8>,
            index_type_1: PhantomData::<u16>,
        };

        assert_eq!(std::mem::size_of_val(&layout), 0);

        assert_eq!(layout.flatten::<[u8; 3]>(&[1, 2, 3]), Some(1024 + 64 + 3));
    }
}
