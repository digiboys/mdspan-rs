use constant::Const;
pub use constant::Constant;
use integer::Integer;

mod private {
    /// Prevents external implementations of [`Extents`].
    pub trait Sealed {}
}

pub type Indices<E> = <<E as Extents>::Rank as Constant>::ArrayOf<<E as Extents>::IndexType>;

pub trait Extents: std::ops::Index<usize, Output = Self::IndexType> + Clone + Eq + private::Sealed {
    type IndexType: Integer;
    type Rank: Constant;
    const RANK: usize;

    fn new(exts: <Self::Rank as Constant>::ArrayOf<Self::IndexType>) -> Self;

    fn iter(&self) -> impl DoubleEndedIterator<Item = Self::IndexType> + ExactSizeIterator;

    fn extent(&self, dimension: usize) -> Self::IndexType {
        assert!(dimension < Self::RANK, "dimension out of bounds");
        self[dimension]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DExtents<I: Integer, const N: usize>([I; N]);

impl<I: Integer, const N: usize> std::ops::Index<usize> for DExtents<I, N> {
    type Output = I;
    fn index(&self, index: usize) -> &I {
        &self.0[index]
    }
}

impl<I: Integer, const N: usize> private::Sealed for DExtents<I, N> {}

impl<I: Integer, const N: usize> Extents for DExtents<I, N> {
    type IndexType = I;
    type Rank = Const<N>;
    const RANK: usize = N;

    fn new(exts: <Self::Rank as Constant>::ArrayOf<Self::IndexType>) -> Self {
        assert!(exts.iter().all(|e| e >= &I::zero()), "extents must be non-negative");
        Self(exts)
    }

    fn iter(&self) -> impl DoubleEndedIterator<Item = Self::IndexType> + ExactSizeIterator {
        self.0.iter().copied()
    }
}

pub type Dims<const N: usize> = DExtents<usize, N>;

#[macro_export]
macro_rules! dextents {
    ($t:ty $(,)?) => {
        DExtents::<$t, 0>::new([])
    };
    ($t:ty, $($x:expr),* $(,)?) => {
        DExtents::<$t, { [$($x as usize),*].len() }>::new([$($x),*])
    };
}

#[macro_export]
macro_rules! dims {
    () => {
        Dims::<0>::new([])
    };
    ($($x:expr),* $(,)?) => {
        Dims::<{ [$($x),*].len() }>::new([$($x),*])
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rank() {
        assert_eq!(<DExtents<usize, 3> as Extents>::RANK, 3);
        assert_eq!(<DExtents<usize, 2> as Extents>::RANK, 2);
    }

    #[test]
    fn test_extent() {
        let extents = Dims::<3>::new([4, 3, 2]);
        assert_eq!(extents.extent(0), 4);
        assert_eq!(extents.extent(1), 3);
        assert_eq!(extents.extent(2), 2);
    }

    #[test]
    #[should_panic(expected = "dimension out of bounds")]
    fn test_extent_out_of_bounds() {
        let extents = Dims::<3>::new([4, 3, 2]);
        extents.extent(3);
    }

    #[test]
    #[should_panic(expected = "extents must be non-negative")]
    fn test_negative_extent() {
        let _ = DExtents::<i32, 3>::new([-1, 3, 2]);
    }

    #[test]
    fn test_iter() {
        let extents = Dims::<3>::new([4, 3, 2]);
        let collected: Vec<usize> = extents.iter().collect();
        assert_eq!(collected, vec![4, 3, 2]);
    }

    #[test]
    fn test_with_u32() {
        let extents = DExtents::<u32, 3>::new([4, 3, 2]);
        assert_eq!(extents.extent(0), 4u32);
        assert_eq!(<DExtents<u32, 3> as Extents>::RANK, 3);

        let collected: Vec<u32> = extents.iter().collect();
        assert_eq!(collected, vec![4u32, 3, 2]);
    }

    #[test]
    fn test_eq() {
        let extents = DExtents::<u32, 3>::new([4, 3, 2]);
        assert_eq!(extents, extents);
    }

    #[test]
    fn test_dims_macro() {
        let extents1 = Dims::<3>::new([4, 3, 2]);
        let extents2 = dims!(4, 3, 2);
        assert_eq!(extents1, extents2);

        let extents3 = Dims::<0>::new([]);
        let extents4 = dims!();
        assert_eq!(extents3, extents4);
    }

    #[test]
    fn test_dextents_macro() {
        let extents1 = DExtents::<i8, 3>::new([4, 3, 2]);
        let extents2 = dextents!(i8, 4, 3, 2);
        assert_eq!(extents1, extents2);

        let extents3 = DExtents::<i16, 0>::new([]);
        let extents4 = dextents!(i16);
        assert_eq!(extents3, extents4);
    }
}
