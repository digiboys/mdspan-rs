use constant::Constant;
use exclusive_product_scan_from::*;
use fill_from_fn::*;
use inner_product::*;
use mod_extents::{Extents, Indices};
use to_usize::ToUsizeExt;

pub trait Mapping {
    type Extents: Extents;

    fn extents(&self) -> &Self::Extents;

    fn stride(&self, dimension: usize) -> usize;

    /// # Safety
    /// The required_span_size should always return the same value.
    unsafe fn required_span_size(&self) -> usize;

    /// # Safety
    /// The value returned by [`Self::to_memory_index`] must be less than the value returned by [`Self::required_span_size`].
    unsafe fn to_memory_index(&self, index: Indices<Self::Extents>) -> usize;
}

type Strides<E> = <<E as Extents>::Rank as Constant>::ArrayOf<usize>;

pub trait Layout {
    type Mapping<E: Extents>;
}

macro_rules! impl_layout_left_or_right {
    ($layout:ident, $mapping:ident, $scan_dir:ty) => {
        struct $layout;

        impl Layout for $layout {
            type Mapping<E: Extents> = $mapping<E>;
        }

        struct $mapping<E>(E);

        impl<E: Extents> Mapping for $mapping<E> {
            type Extents = E;

            fn extents(&self) -> &Self::Extents {
                &self.0
            }

            fn stride(&self, dimension: usize) -> usize {
                assert!(dimension < E::RANK, "dimension out of bounds");

                let iter = self.extents().iter().to_usize();
                if <$scan_dir as direction::IsForward>::VALUE {
                    iter.take(dimension).product()
                } else {
                    iter.skip(dimension + 1).product()
                }
            }

            unsafe fn required_span_size(&self) -> usize {
                self.extents().iter().to_usize().product()
            }

            unsafe fn to_memory_index(&self, indices: Indices<E>) -> usize {
                debug_assert!(std::iter::zip(indices, self.extents().iter()).all(|(i, e)| i < e));

                // SAFETY: `Strides<E>` is `[usize; N]`, which is a `Copy` type.
                // `Copy` types cannot implement `Drop`, so no destructor will run on
                // the uninitialized memory. Every element is written by `fill_from_fn`
                // before `strides` is read, so no uninitialized value is ever observed.
                let mut strides: Strides<E> = unsafe { core::mem::MaybeUninit::uninit().assume_init() };

                use rev_if::RevIfExt;
                fill_from_fn(
                    strides.as_mut().iter_mut().rev_if::<$scan_dir>(),
                    exclusive_product_scan_from(self.extents().iter().to_usize().rev_if::<$scan_dir>()),
                );

                inner_product(indices.into_iter().to_usize(), strides)
            }
        }
    };
}

impl_layout_left_or_right!(LayoutLeft, LayoutLeftMapping, direction::Forward);
impl_layout_left_or_right!(LayoutRight, LayoutRightMapping, direction::Reverse);

#[cfg(test)]
mod tests {
    use mod_extents::{Dims, dims};

    use super::*;

    #[test]
    fn test_layout_left_mapping() {
        let mapping = LayoutLeftMapping(dims!(2, 2));

        unsafe {
            assert_eq!(0, mapping.to_memory_index([0, 0]));
            assert_eq!(1, mapping.to_memory_index([1, 0]));
            assert_eq!(2, mapping.to_memory_index([0, 1]));
            assert_eq!(3, mapping.to_memory_index([1, 1]));

            assert_eq!(4, mapping.required_span_size());
        }
    }

    #[test]
    #[should_panic(expected = "dimension out of bounds")]
    fn test_layout_left_mapping_stride_out_of_bounds() {
        let mapping = LayoutLeftMapping(dims!(4, 3, 2));
        mapping.stride(3);
    }

    #[test]
    fn test_layout_right_mapping() {
        let mapping = LayoutRightMapping(dims!(2, 2));

        unsafe {
            assert_eq!(0, mapping.to_memory_index([0, 0]));
            assert_eq!(1, mapping.to_memory_index([0, 1]));
            assert_eq!(2, mapping.to_memory_index([1, 0]));
            assert_eq!(3, mapping.to_memory_index([1, 1]));

            assert_eq!(4, mapping.required_span_size());
        }
    }

    #[test]
    #[should_panic(expected = "dimension out of bounds")]
    fn test_layout_right_mapping_stride_out_of_bounds() {
        let mapping = LayoutRightMapping(dims!(4, 3, 2));
        mapping.stride(3);
    }
}
