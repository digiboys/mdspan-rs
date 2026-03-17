// pub use mod_accessor::*;
// pub use mod_layout::*;
// pub use mod_mdspan::*;

use core::{
    iter::Sum,
    ops::{Add, Mul},
};
use std::marker::PhantomData;

pub trait Layout {
    type Extent;
    type Stride;

    fn flatten<TIndices>(&self, indices: &TIndices) -> Option<Self::Stride>
    where
        for<'a> &'a TIndices: IntoIterator<Item = &'a Self::Stride>;
}

pub struct AxialLayout<TExtents, TStrides, Textent, TStride> {
    extents: TExtents,
    strides: TStrides,
    _extent_type: PhantomData<Textent>,
    _stride_type: PhantomData<TStride>,
}

impl<TExtents, TStrides, TExtent, TStride> AxialLayout<TExtents, TStrides, TExtent, TStride> {
    pub fn new(extents: TExtents, strides: TStrides) -> Self
    where
        for<'a> &'a TExtents: IntoIterator<Item = &'a TExtent>,
        for<'a> &'a TStrides: IntoIterator<Item = &'a TStride>,
    {
        Self {
            extents,
            strides,
            _extent_type: PhantomData,
            _stride_type: PhantomData,
        }
    }
}

impl<TExtents, TStrides, Extent, Stride> Layout for AxialLayout<TExtents, TStrides, Extent, Stride>
where
    for<'a> &'a TExtents: IntoIterator<Item = &'a Extent>,
    for<'a> &'a TStrides: IntoIterator<Item = &'a Stride>,
    Extent: Copy + Into<Stride>,
    Stride: Copy + Ord + Add<Output = Stride> + Mul<Output = Stride> + Sum,
{
    type Extent = Extent;
    type Stride = Stride;

    fn flatten<TIndices>(&self, indices: &TIndices) -> Option<Self::Stride>
    where
        for<'a> &'a TIndices: IntoIterator<Item = &'a Self::Stride>,
    {
        if std::iter::zip(indices, &self.extents).all(|(&i, &e)| i < e.into()) {
            Some(std::iter::zip(indices, &self.strides).map(|(&i, &s)| i * s).sum())
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

    #[test]
    fn x() {
        let layout = AxialLayout::new(ChunkExtents, ChunkStrides);
        assert_eq!(std::mem::size_of_val(&layout), 0);
        assert_eq!(layout.flatten(&[1, 2, 3]), Some(1024 + 64 + 3));

        let layout = AxialLayout::new([32u16; 3], ChunkStrides);
        assert_eq!(std::mem::size_of_val(&layout), 6);
        assert_eq!(layout.flatten(&[1, 2, 3]), Some(1024 + 64 + 3));
    }
}
