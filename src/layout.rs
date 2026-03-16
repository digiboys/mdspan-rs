use core::fmt;

use mdspan_extents::ExtentsType;

#[derive(Debug, Clone, Copy, Default)]
pub struct LayoutRight;

#[derive(Debug, Clone, Copy, Default)]
pub struct LayoutLeft;

#[derive(Debug, Clone, Copy, Default)]
pub struct LayoutStride;

pub trait Mapping: Clone + fmt::Debug {
    type Extents: ExtentsType;
    type Layout: Clone + Copy + fmt::Debug;

    const IS_ALWAYS_UNIQUE: bool;
    const IS_ALWAYS_EXHAUSTIVE: bool;
    const IS_ALWAYS_STRIDED: bool;

    fn extents(&self) -> &Self::Extents;
    fn required_span_size(&self) -> usize;
    fn offset_unchecked(&self, indices: &[usize]) -> usize;
    fn stride(&self, dimension: usize) -> Option<usize>;

    fn rank(&self) -> usize {
        Self::Extents::RANK
    }

    fn is_unique(&self) -> bool {
        Self::IS_ALWAYS_UNIQUE
    }

    fn is_exhaustive(&self) -> bool {
        Self::IS_ALWAYS_EXHAUSTIVE
    }

    fn is_strided(&self) -> bool {
        Self::IS_ALWAYS_STRIDED
    }

    fn check_indices(&self, indices: &[usize]) -> bool {
        indices.len() == Self::Extents::RANK
            && indices
                .iter()
                .copied()
                .enumerate()
                .all(|(dimension, index)| index < self.extents().extent(dimension))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayoutRightMapping<E> {
    extents: E,
}

impl<E: ExtentsType> LayoutRightMapping<E> {
    pub fn new(extents: E) -> Self {
        Self { extents }
    }
}

impl<E: ExtentsType> Mapping for LayoutRightMapping<E> {
    type Extents = E;
    type Layout = LayoutRight;

    const IS_ALWAYS_UNIQUE: bool = true;
    const IS_ALWAYS_EXHAUSTIVE: bool = true;
    const IS_ALWAYS_STRIDED: bool = true;

    fn extents(&self) -> &Self::Extents {
        &self.extents
    }

    fn required_span_size(&self) -> usize {
        self.extents.size()
    }

    fn offset_unchecked(&self, indices: &[usize]) -> usize {
        let mut offset = 0usize;
        let mut stride = 1usize;
        for dimension in (0..E::RANK).rev() {
            offset += indices[dimension] * stride;
            stride *= self.extents.extent(dimension);
        }
        offset
    }

    fn stride(&self, dimension: usize) -> Option<usize> {
        if dimension >= E::RANK {
            return None;
        }
        let mut stride = 1usize;
        for next in (dimension + 1)..E::RANK {
            stride *= self.extents.extent(next);
        }
        Some(stride)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayoutLeftMapping<E> {
    extents: E,
}

impl<E: ExtentsType> LayoutLeftMapping<E> {
    pub fn new(extents: E) -> Self {
        Self { extents }
    }
}

impl<E: ExtentsType> Mapping for LayoutLeftMapping<E> {
    type Extents = E;
    type Layout = LayoutLeft;

    const IS_ALWAYS_UNIQUE: bool = true;
    const IS_ALWAYS_EXHAUSTIVE: bool = true;
    const IS_ALWAYS_STRIDED: bool = true;

    fn extents(&self) -> &Self::Extents {
        &self.extents
    }

    fn required_span_size(&self) -> usize {
        self.extents.size()
    }

    fn offset_unchecked(&self, indices: &[usize]) -> usize {
        let mut offset = 0usize;
        let mut stride = 1usize;
        for dimension in 0..E::RANK {
            offset += indices[dimension] * stride;
            stride *= self.extents.extent(dimension);
        }
        offset
    }

    fn stride(&self, dimension: usize) -> Option<usize> {
        if dimension >= E::RANK {
            return None;
        }
        let mut stride = 1usize;
        for prior in 0..dimension {
            stride *= self.extents.extent(prior);
        }
        Some(stride)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutStrideError {
    NonPositiveStride { dimension: usize },
    RankMismatch { expected: usize, actual: usize },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayoutStrideMapping<E> {
    extents: E,
    strides: Vec<usize>,
}

impl<E: ExtentsType> LayoutStrideMapping<E> {
    pub fn new(extents: E, strides: impl Into<Vec<usize>>) -> Result<Self, LayoutStrideError> {
        let strides = strides.into();
        if strides.len() != E::RANK {
            return Err(LayoutStrideError::RankMismatch {
                expected: E::RANK,
                actual: strides.len(),
            });
        }
        for (dimension, stride) in strides.iter().copied().enumerate() {
            if stride == 0 {
                return Err(LayoutStrideError::NonPositiveStride { dimension });
            }
        }
        Ok(Self { extents, strides })
    }

    pub fn strides(&self) -> &[usize] {
        &self.strides
    }
}

impl<E: ExtentsType> Mapping for LayoutStrideMapping<E> {
    type Extents = E;
    type Layout = LayoutStride;

    const IS_ALWAYS_UNIQUE: bool = false;
    const IS_ALWAYS_EXHAUSTIVE: bool = false;
    const IS_ALWAYS_STRIDED: bool = true;

    fn extents(&self) -> &Self::Extents {
        &self.extents
    }

    fn required_span_size(&self) -> usize {
        if E::RANK == 0 {
            return 1;
        }
        if self.extents.extents().contains(&0) {
            return 0;
        }
        1 + self
            .strides
            .iter()
            .copied()
            .enumerate()
            .map(|(dimension, stride)| (self.extents.extent(dimension) - 1) * stride)
            .sum::<usize>()
    }

    fn offset_unchecked(&self, indices: &[usize]) -> usize {
        self.strides
            .iter()
            .copied()
            .enumerate()
            .map(|(dimension, stride)| indices[dimension] * stride)
            .sum()
    }

    fn stride(&self, dimension: usize) -> Option<usize> {
        self.strides.get(dimension).copied()
    }

    fn is_unique(&self) -> bool {
        if E::RANK <= 1 {
            return true;
        }

        let mut order = (0..E::RANK).collect::<Vec<_>>();
        order.sort_by_key(|&dimension| self.strides[dimension]);

        for pair in order.windows(2) {
            let lhs = pair[0];
            let rhs = pair[1];
            let lower_extent = self.extents.extent(lhs);
            if self.strides[rhs] < self.strides[lhs].saturating_mul(lower_extent.max(1)) {
                return false;
            }
        }

        true
    }

    fn is_exhaustive(&self) -> bool {
        self.is_unique() && self.required_span_size() == self.extents.size()
    }
}
