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

fn extents_to_strides_layout_right<const N: usize>(extents: [usize; N]) -> [usize; N] {
    let mut stride = 1;
    let mut strides = core::array::from_fn(|offset| {
        let dimension = N - 1 - offset;
        let current = stride;
        stride *= extents[dimension];
        current
    });
    strides.reverse();
    strides
}

pub trait Layout {
    type Mapping<TExtents>;
}

struct LayoutRight;

impl Layout for LayoutRight {
    type Mapping<TExtents> = LayoutRightMapping<TExtents>;
}

struct LayoutRightMapping<TExtents>(TExtents);

impl<const N: usize> Mapping for LayoutRightMapping<[usize; N]> {
    type Extents = [usize; N];
    type Indices = [usize; N];

    fn extents(&self) -> &Self::Extents {
        &self.0
    }

    fn stride(&self, dimension: usize) -> usize {
        assert!(dimension < self.0.len(), "dimension out of bounds");
        self.0[(dimension + 1)..].iter().copied().product()
    }

    unsafe fn required_span_size(&self) -> usize {
        self.0.iter().copied().product()
    }

    unsafe fn to_memory_index(&self, indices: Self::Indices) -> usize {
        debug_assert!(std::iter::zip(indices, self.0).all(|(i, e)| i < e));
        let strides = extents_to_strides_layout_right(self.0);
        std::iter::zip(indices, strides).map(|(a, b)| a * b).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
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
    fn test_layout_right_mapping_stride() {
        let mapping = LayoutRightMapping([2, 2]);

        assert!(
            std::iter::zip(
                0..mapping.extents().len(),
                extents_to_strides_layout_right(*mapping.extents())
            )
            .all(|(i, s)| mapping.stride(i) == s)
        );
    }

    #[test]
    #[should_panic(expected = "dimension out of bounds")]
    fn test_layout_right_mapping_stride_out_of_bounds() {
        let mapping = LayoutRightMapping([4, 3, 2]);
        mapping.stride(3);
    }
}
