use core::fmt;

pub const MAX_RANK: usize = 12;
pub const DYNAMIC_EXTENT: usize = usize::MAX;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtentsError {
    RankTooLarge {
        rank: usize,
        max_rank: usize,
    },
    StaticMismatch {
        dimension: usize,
        expected: usize,
        actual: usize,
    },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Extents<
    const R: usize,
    const E0: usize = DYNAMIC_EXTENT,
    const E1: usize = DYNAMIC_EXTENT,
    const E2: usize = DYNAMIC_EXTENT,
    const E3: usize = DYNAMIC_EXTENT,
    const E4: usize = DYNAMIC_EXTENT,
    const E5: usize = DYNAMIC_EXTENT,
    const E6: usize = DYNAMIC_EXTENT,
    const E7: usize = DYNAMIC_EXTENT,
    const E8: usize = DYNAMIC_EXTENT,
    const E9: usize = DYNAMIC_EXTENT,
    const E10: usize = DYNAMIC_EXTENT,
    const E11: usize = DYNAMIC_EXTENT,
> {
    values: [usize; R],
}

impl<
    const R: usize,
    const E0: usize,
    const E1: usize,
    const E2: usize,
    const E3: usize,
    const E4: usize,
    const E5: usize,
    const E6: usize,
    const E7: usize,
    const E8: usize,
    const E9: usize,
    const E10: usize,
    const E11: usize,
> fmt::Debug for Extents<R, E0, E1, E2, E3, E4, E5, E6, E7, E8, E9, E10, E11>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Extents")
            .field("rank", &R)
            .field("values", &self.values)
            .finish()
    }
}

pub type DExtents<const R: usize> = Extents<
    R,
    DYNAMIC_EXTENT,
    DYNAMIC_EXTENT,
    DYNAMIC_EXTENT,
    DYNAMIC_EXTENT,
    DYNAMIC_EXTENT,
    DYNAMIC_EXTENT,
    DYNAMIC_EXTENT,
    DYNAMIC_EXTENT,
    DYNAMIC_EXTENT,
    DYNAMIC_EXTENT,
    DYNAMIC_EXTENT,
    DYNAMIC_EXTENT,
>;

pub trait ExtentsType: Clone + fmt::Debug + PartialEq + Eq {
    const RANK: usize;

    fn extent(&self, dimension: usize) -> usize;
    fn static_extent(dimension: usize) -> usize;
    fn extents(&self) -> &[usize];

    fn rank(&self) -> usize {
        Self::RANK
    }

    fn rank_dynamic() -> usize {
        (0..Self::RANK)
            .filter(|&dimension| Self::static_extent(dimension) == DYNAMIC_EXTENT)
            .count()
    }

    fn size(&self) -> usize {
        if Self::RANK == 0 {
            1
        } else {
            self.extents().iter().copied().fold(1usize, usize::saturating_mul)
        }
    }
}

impl<
    const R: usize,
    const E0: usize,
    const E1: usize,
    const E2: usize,
    const E3: usize,
    const E4: usize,
    const E5: usize,
    const E6: usize,
    const E7: usize,
    const E8: usize,
    const E9: usize,
    const E10: usize,
    const E11: usize,
> Extents<R, E0, E1, E2, E3, E4, E5, E6, E7, E8, E9, E10, E11>
{
    pub const STATIC_EXTENTS: [usize; MAX_RANK] = [E0, E1, E2, E3, E4, E5, E6, E7, E8, E9, E10, E11];

    pub fn new(values: [usize; R]) -> Result<Self, ExtentsError> {
        if R > MAX_RANK {
            return Err(ExtentsError::RankTooLarge {
                rank: R,
                max_rank: MAX_RANK,
            });
        }

        for (dimension, actual) in values.iter().copied().enumerate() {
            let expected = Self::STATIC_EXTENTS[dimension];
            if expected != DYNAMIC_EXTENT && expected != actual {
                return Err(ExtentsError::StaticMismatch {
                    dimension,
                    expected,
                    actual,
                });
            }
        }

        Ok(Self { values })
    }

    pub const fn from_resolved(values: [usize; R]) -> Self {
        Self { values }
    }

    pub fn extent(&self, dimension: usize) -> usize {
        self.values[dimension]
    }

    pub fn extents(&self) -> &[usize] {
        &self.values
    }
}

impl<
    const R: usize,
    const E0: usize,
    const E1: usize,
    const E2: usize,
    const E3: usize,
    const E4: usize,
    const E5: usize,
    const E6: usize,
    const E7: usize,
    const E8: usize,
    const E9: usize,
    const E10: usize,
    const E11: usize,
> ExtentsType for Extents<R, E0, E1, E2, E3, E4, E5, E6, E7, E8, E9, E10, E11>
{
    const RANK: usize = R;

    fn extent(&self, dimension: usize) -> usize {
        self.values[dimension]
    }

    fn static_extent(dimension: usize) -> usize {
        Self::STATIC_EXTENTS[dimension]
    }

    fn extents(&self) -> &[usize] {
        &self.values
    }
}
