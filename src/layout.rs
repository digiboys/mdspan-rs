use exclusive_product_scan::*;
use inner_product::*;

pub trait Mapping {
    type Extents;
    type Indices;

    fn extents(&self) -> &Self::Extents;

    fn stride(&self, dimension: usize) -> usize;

    /// # Safety
    /// The required_span_size should always return the same value.
    unsafe fn required_span_size(&self) -> usize;

    /// # Safety
    /// The value returned by [`Self::to_memory_index`] must be less than the value returned by [`Self::required_span_size`].
    unsafe fn to_memory_index(&self, index: Self::Indices) -> usize;
}

pub trait Layout {
    type Mapping<TExtents>;
}

macro_rules! impl_layout_left_or_right {
    ($layout:ident, $mapping:ident, $scan_dir:ty) => {
        struct $layout;

        impl Layout for $layout {
            type Mapping<E> = $mapping<E>;
        }

        struct $mapping<E>(E);

        impl<const N: usize> Mapping for $mapping<[usize; N]> {
            type Extents = [usize; N];
            type Indices = [usize; N];

            fn extents(&self) -> &Self::Extents {
                &self.0
            }

            fn stride(&self, dimension: usize) -> usize {
                assert!(dimension < self.0.len(), "dimension out of bounds");
                let slice = if <$scan_dir as direction::IsForward>::VALUE {
                    &self.0[..dimension]
                } else {
                    &self.0[(dimension + 1)..]
                };
                slice.iter().copied().product()
            }

            unsafe fn required_span_size(&self) -> usize {
                self.0.iter().copied().product()
            }

            unsafe fn to_memory_index(&self, indices: Self::Indices) -> usize {
                debug_assert!(std::iter::zip(indices, self.0).all(|(i, e)| i < e));
                let strides = exclusive_product_scan::<$scan_dir, N, _>(self.0);
                inner_product(indices, strides)
            }
        }
    };
}

impl_layout_left_or_right!(LayoutLeft, LayoutLeftMapping, direction::Forward);
impl_layout_left_or_right!(LayoutRight, LayoutRightMapping, direction::Reverse);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[inline(never)]
    fn test_layout_left_mapping() {
        let mapping = LayoutLeftMapping([2, 2]);

        unsafe {
            assert_eq!(0, mapping.to_memory_index([0, 0]));
            assert_eq!(1, mapping.to_memory_index([1, 0]));
            assert_eq!(2, mapping.to_memory_index([0, 1]));
            assert_eq!(3, mapping.to_memory_index([1, 1]));

            assert_eq!(4, mapping.required_span_size());
        }
    }

    #[test]
    #[inline(never)]
    #[should_panic(expected = "dimension out of bounds")]
    fn test_layout_left_mapping_stride_out_of_bounds() {
        let mapping = LayoutLeftMapping([4, 3, 2]);
        mapping.stride(3);
    }

    #[test]
    #[inline(never)]
    fn test_layout_right_mapping() {
        let mapping = LayoutRightMapping([2, 2]);

        unsafe {
            assert_eq!(0, mapping.to_memory_index([0, 0]));
            assert_eq!(1, mapping.to_memory_index([0, 1]));
            assert_eq!(2, mapping.to_memory_index([1, 0]));
            assert_eq!(3, mapping.to_memory_index([1, 1]));

            assert_eq!(4, mapping.required_span_size());
        }
    }

    #[test]
    #[inline(never)]
    #[should_panic(expected = "dimension out of bounds")]
    fn test_layout_right_mapping_stride_out_of_bounds() {
        let mapping = LayoutRightMapping([4, 3, 2]);
        mapping.stride(3);
    }
}
