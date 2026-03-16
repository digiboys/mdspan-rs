//! Rust `mdspan` core modeled on the C++ standard wording in
//! <https://eel.is/c++draft/views#multidim>.
//!
//! Deviations from C++ are deliberate where Rust needs stronger aliasing and
//! lifetime guarantees:
//! - extents support ranks up to `MAX_RANK = 12` through const generics
//! - checked indexing uses slices instead of parameter packs
//! - mutable spans reject non-unique mappings at construction time

pub use mdspan_accessor::{Accessor, AccessorMut, ConstantAccessor, DefaultAccessor, ScaledAccessor};
pub use mdspan_extents::{DExtents, DYNAMIC_EXTENT, Extents, ExtentsError, ExtentsType, MAX_RANK};
pub use mdspan_layout::{
    LayoutLeft, LayoutLeftMapping, LayoutRight, LayoutRightMapping, LayoutStride, LayoutStrideError,
    LayoutStrideMapping, Mapping,
};
pub use mdspan_span::{MdSpan, MdSpanBuilder, MdSpanError, MdSpanMut, MdSpanMutBuilder};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dynamic_layout_right_builder() {
        let data = [0, 1, 2, 3, 4, 5];
        let extents = DExtents::<2>::new([2, 3]).unwrap();
        let span = MdSpanBuilder::new(&data, extents).layout_right().unwrap();

        assert_eq!(span.rank(), 2);
        assert_eq!(span.extent(0), 2);
        assert_eq!(span.extent(1), 3);
        assert_eq!(span.get(&[1, 2]), Some(&5));
        assert_eq!(span.required_span_size(), 6);
        assert_eq!(span.stride(0), Some(3));
        assert_eq!(span.stride(1), Some(1));
    }

    #[test]
    fn mixed_static_extents_layout_left() {
        type MatrixExtents = Extents<2, 2, DYNAMIC_EXTENT>;
        let data = [0, 1, 2, 3, 4, 5];
        let extents = MatrixExtents::new([2, 3]).unwrap();
        let span = MdSpanBuilder::new(&data, extents).layout_left().unwrap();

        assert_eq!(MatrixExtents::RANK, 2);
        assert_eq!(MatrixExtents::static_extent(0), 2);
        assert_eq!(MatrixExtents::static_extent(1), DYNAMIC_EXTENT);
        assert_eq!(span.get(&[1, 2]), Some(&5));
        assert_eq!(span.stride(0), Some(1));
        assert_eq!(span.stride(1), Some(2));
    }

    #[test]
    fn layout_stride_queries_and_offsets() {
        let data = [10, 11, 12, 13, 14, 15, 16, 17];
        let extents = DExtents::<2>::new([2, 3]).unwrap();
        let span = MdSpan::<i32, LayoutRightMapping<DExtents<1>>>::builder(&data, extents)
            .layout_stride([1, 3])
            .unwrap();

        assert_eq!(span.get(&[1, 2]), Some(&17));
        assert_eq!(span.required_span_size(), 8);
        assert!(span.is_unique());
        assert!(!span.is_exhaustive());
    }

    #[test]
    fn constant_accessor_ignores_indices() {
        let extents = DExtents::<2>::new([2, 2]).unwrap();
        let span = MdSpan::<i32, LayoutRightMapping<DExtents<1>>>::constant(7, extents).unwrap();

        assert_eq!(span.get(&[0, 0]), Some(7));
        assert_eq!(span.get(&[1, 1]), Some(7));
    }

    #[test]
    fn scaled_accessor_returns_scaled_values() {
        let data = [1, 2, 3, 4];
        let extents = DExtents::<2>::new([2, 2]).unwrap();
        let span = MdSpan::<i32, LayoutRightMapping<DExtents<1>>>::builder(&data, extents)
            .scaled(10)
            .layout_right()
            .unwrap();

        assert_eq!(span.get(&[0, 1]), Some(20));
        assert_eq!(span.get(&[1, 1]), Some(40));
    }

    #[test]
    fn mutable_access_requires_unique_mapping() {
        let mut data = [0, 1, 2, 3];
        let extents = DExtents::<2>::new([2, 2]).unwrap();

        let err = MdSpanMut::<i32, LayoutRightMapping<DExtents<1>>>::builder_mut(&mut data, extents)
            .layout_stride([1, 1])
            .unwrap_err();
        assert_eq!(err, MdSpanError::NonUniqueMapping);
    }

    #[test]
    fn mutable_access_updates_underlying_slice() {
        let mut data = [0, 1, 2, 3, 4, 5];
        let extents = DExtents::<2>::new([2, 3]).unwrap();
        let mut span = MdSpanMut::<i32, LayoutRightMapping<DExtents<1>>>::builder_mut(&mut data, extents)
            .layout_right()
            .unwrap();

        *span.get_mut(&[1, 1]).unwrap() = 99;
        assert_eq!(span.get(&[1, 1]), Some(&99));
        assert_eq!(data[4], 99);
    }
}
