pub trait Mapping {
    type Extents;
    type Indices;

    // type Layout;

    // const IS_ALWAYS_UNIQUE: bool;
    // const IS_ALWAYS_EXHAUSTIVE: bool;
    // const IS_ALWAYS_STRIDED: bool;

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
    core::array::from_fn(|offset| {
        let dimension = N - 1 - offset;
        let current = stride;
        stride *= extents[dimension];
        current
    })
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
