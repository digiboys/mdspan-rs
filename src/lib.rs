// pub use mod_accessor::*;
// pub use mod_layout::*;
// pub use mod_mdspan::*;

pub trait Layout {
    type IndicesElement;
    type Index;

    fn flatten<TIndices>(&self, indices: &TIndices) -> Option<Self::Index>
    where
        for<'a> &'a TIndices: IntoIterator<Item = &'a Self::IndicesElement>;
}

pub struct AxialLayout<TExtents, TStrides> {
    extents: TExtents,
    strides: TStrides,
}

impl<TExtents, TStrides> Layout for AxialLayout<TExtents, TStrides>
where
    for<'a> &'a TExtents: IntoIterator<Item = &'a u8>,
    for<'a> &'a TStrides: IntoIterator<Item = &'a u16>,
{
    type IndicesElement = u8;
    type Index = u16;

    fn flatten<TIndices>(&self, indices: &TIndices) -> Option<Self::Index>
    where
        for<'a> &'a TIndices: IntoIterator<Item = &'a Self::IndicesElement>,
    {
        if std::iter::zip(indices, &self.extents).all(|(&i, &e)| i < e) {
            Some(
                std::iter::zip(indices, &self.strides)
                    .map(|(&i, &s)| i as u16 * s)
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

    #[test]
    fn x() {
        let layout = AxialLayout {
            extents: ChunkExtents,
            strides: ChunkStrides,
        };

        assert_eq!(std::mem::size_of_val(&layout), 0);

        assert_eq!(layout.flatten(&[1, 2, 3]), Some(1024 + 64 + 3));
    }
}
