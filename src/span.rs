use core::{fmt, marker::PhantomData, ptr::NonNull};

use mdspan_accessor::{Accessor, AccessorMut, ConstantAccessor, DefaultAccessor, ScaledAccessor};
use mdspan_extents::ExtentsType;
use mdspan_layout::{LayoutLeftMapping, LayoutRightMapping, LayoutStrideError, LayoutStrideMapping, Mapping};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MdSpanError {
    IndexRankMismatch {
        expected: usize,
        actual: usize,
    },
    IndexOutOfBounds {
        dimension: usize,
        index: usize,
        extent: usize,
    },
    InsufficientElements {
        required: usize,
        available: usize,
    },
    NonUniqueMapping,
    LayoutStride(LayoutStrideError),
}

impl From<LayoutStrideError> for MdSpanError {
    fn from(value: LayoutStrideError) -> Self {
        Self::LayoutStride(value)
    }
}

#[derive(Clone, Debug)]
pub struct MdSpan<'a, T, M, A = DefaultAccessor<T>>
where
    M: Mapping,
    A: Accessor<Element = T>,
{
    data_handle: A::Pointer,
    mapping: M,
    accessor: A,
    _marker: PhantomData<&'a T>,
}

#[derive(Debug)]
pub struct MdSpanMut<'a, T, M, A = DefaultAccessor<T>>
where
    M: Mapping,
    A: AccessorMut<Element = T>,
{
    data_handle: A::Pointer,
    mapping: M,
    accessor: A,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T, M, A> MdSpan<'a, T, M, A>
where
    M: Mapping,
    A: Accessor<Element = T>,
{
    pub unsafe fn from_raw_parts(data_handle: A::Pointer, mapping: M, accessor: A) -> Self {
        Self {
            data_handle,
            mapping,
            accessor,
            _marker: PhantomData,
        }
    }

    pub fn mapping(&self) -> &M {
        &self.mapping
    }

    pub fn accessor(&self) -> &A {
        &self.accessor
    }

    pub fn rank(&self) -> usize {
        self.mapping.rank()
    }

    pub fn extent(&self, dimension: usize) -> usize {
        self.mapping.extents().extent(dimension)
    }

    pub fn extents(&self) -> &M::Extents {
        self.mapping.extents()
    }

    pub fn required_span_size(&self) -> usize {
        self.mapping.required_span_size()
    }

    pub fn stride(&self, dimension: usize) -> Option<usize> {
        self.mapping.stride(dimension)
    }

    pub fn is_unique(&self) -> bool {
        self.mapping.is_unique()
    }

    pub fn is_exhaustive(&self) -> bool {
        self.mapping.is_exhaustive()
    }

    pub fn is_strided(&self) -> bool {
        self.mapping.is_strided()
    }

    pub fn get(&self, indices: &[usize]) -> Option<A::Reference<'_>> {
        self.validate_indices(indices).ok()?;
        unsafe { Some(self.get_unchecked(indices)) }
    }

    pub unsafe fn get_unchecked(&self, indices: &[usize]) -> A::Reference<'_> {
        let offset = self.mapping.offset_unchecked(indices);
        self.accessor.access(&self.data_handle, offset)
    }

    pub fn validate_indices(&self, indices: &[usize]) -> Result<(), MdSpanError> {
        if indices.len() != self.rank() {
            return Err(MdSpanError::IndexRankMismatch {
                expected: self.rank(),
                actual: indices.len(),
            });
        }

        for (dimension, index) in indices.iter().copied().enumerate() {
            let extent = self.extent(dimension);
            if index >= extent {
                return Err(MdSpanError::IndexOutOfBounds {
                    dimension,
                    index,
                    extent,
                });
            }
        }

        Ok(())
    }
}

impl<'a, T, M, A> Clone for MdSpanMut<'a, T, M, A>
where
    M: Mapping + Clone,
    A: AccessorMut<Element = T> + Clone,
{
    fn clone(&self) -> Self {
        Self {
            data_handle: self.data_handle.clone(),
            mapping: self.mapping.clone(),
            accessor: self.accessor.clone(),
            _marker: PhantomData,
        }
    }
}

impl<'a, T, M, A> MdSpanMut<'a, T, M, A>
where
    M: Mapping,
    A: AccessorMut<Element = T>,
{
    pub unsafe fn from_raw_parts(data_handle: A::Pointer, mapping: M, accessor: A) -> Result<Self, MdSpanError> {
        if !mapping.is_unique() {
            return Err(MdSpanError::NonUniqueMapping);
        }

        Ok(Self {
            data_handle,
            mapping,
            accessor,
            _marker: PhantomData,
        })
    }

    pub fn mapping(&self) -> &M {
        &self.mapping
    }

    pub fn get(&self, indices: &[usize]) -> Option<A::Reference<'_>> {
        self.validate_indices(indices).ok()?;
        let offset = self.mapping.offset_unchecked(indices);
        Some(self.accessor.access(&self.data_handle, offset))
    }

    pub fn get_mut(&mut self, indices: &[usize]) -> Option<A::MutReference<'_>> {
        self.validate_indices(indices).ok()?;
        unsafe { Some(self.get_unchecked_mut(indices)) }
    }

    pub unsafe fn get_unchecked_mut(&mut self, indices: &[usize]) -> A::MutReference<'_> {
        let offset = self.mapping.offset_unchecked(indices);
        unsafe { self.accessor.access_mut(&mut self.data_handle, offset) }
    }

    pub fn validate_indices(&self, indices: &[usize]) -> Result<(), MdSpanError> {
        if indices.len() != self.mapping.rank() {
            return Err(MdSpanError::IndexRankMismatch {
                expected: self.mapping.rank(),
                actual: indices.len(),
            });
        }

        for (dimension, index) in indices.iter().copied().enumerate() {
            let extent = self.mapping.extents().extent(dimension);
            if index >= extent {
                return Err(MdSpanError::IndexOutOfBounds {
                    dimension,
                    index,
                    extent,
                });
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct MdSpanBuilder<'a, T, E, A = DefaultAccessor<T>>
where
    A: Accessor<Element = T>,
{
    handle: A::Pointer,
    available_len: Option<usize>,
    extents: E,
    accessor: A,
    _marker: PhantomData<&'a T>,
}

#[derive(Clone, Debug)]
pub struct MdSpanMutBuilder<'a, T, E, A = DefaultAccessor<T>>
where
    A: AccessorMut<Element = T>,
{
    handle: A::Pointer,
    available_len: usize,
    extents: E,
    accessor: A,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T> MdSpan<'a, T, LayoutRightMapping<mdspan_extents::DExtents<1>>, DefaultAccessor<T>> {
    pub fn from_slice(slice: &'a [T]) -> Self {
        let extents = mdspan_extents::DExtents::<1>::from_resolved([slice.len()]);
        let handle = NonNull::from(slice).cast::<T>();
        unsafe { Self::from_raw_parts(handle, LayoutRightMapping::new(extents), DefaultAccessor::default()) }
    }
}

impl<'a, T> MdSpanMut<'a, T, LayoutRightMapping<mdspan_extents::DExtents<1>>, DefaultAccessor<T>> {
    pub fn from_slice_mut(slice: &'a mut [T]) -> Self {
        let extents = mdspan_extents::DExtents::<1>::from_resolved([slice.len()]);
        let handle = NonNull::from(slice).cast::<T>();
        unsafe {
            Self::from_raw_parts(handle, LayoutRightMapping::new(extents), DefaultAccessor::default())
                .expect("layout_right is always unique")
        }
    }
}

impl<'a, T, E> MdSpanBuilder<'a, T, E, DefaultAccessor<T>>
where
    E: ExtentsType,
{
    pub fn new(slice: &'a [T], extents: E) -> Self {
        Self {
            handle: NonNull::from(slice).cast::<T>(),
            available_len: Some(slice.len()),
            extents,
            accessor: DefaultAccessor::default(),
            _marker: PhantomData,
        }
    }

    pub fn scaled(self, scale: T) -> MdSpanBuilder<'a, T, E, ScaledAccessor<T>>
    where
        T: Copy + fmt::Debug + core::ops::Mul<Output = T>,
    {
        MdSpanBuilder {
            handle: self.handle,
            available_len: self.available_len,
            extents: self.extents,
            accessor: ScaledAccessor::new(scale),
            _marker: PhantomData,
        }
    }
}

impl<'a, T, E, A> MdSpanBuilder<'a, T, E, A>
where
    E: ExtentsType,
    A: Accessor<Element = T>,
{
    pub fn with_accessor<Next>(self, accessor: Next) -> MdSpanBuilder<'a, T, E, Next>
    where
        Next: Accessor<Element = T, Pointer = A::Pointer>,
    {
        MdSpanBuilder {
            handle: self.handle,
            available_len: self.available_len,
            extents: self.extents,
            accessor,
            _marker: PhantomData,
        }
    }

    pub fn layout_right(self) -> Result<MdSpan<'a, T, LayoutRightMapping<E>, A>, MdSpanError> {
        let Self {
            handle,
            available_len,
            extents,
            accessor,
            ..
        } = self;
        let mapping = LayoutRightMapping::new(extents);
        Self::finish_parts(handle, available_len, accessor, mapping)
    }

    pub fn layout_left(self) -> Result<MdSpan<'a, T, LayoutLeftMapping<E>, A>, MdSpanError> {
        let Self {
            handle,
            available_len,
            extents,
            accessor,
            ..
        } = self;
        let mapping = LayoutLeftMapping::new(extents);
        Self::finish_parts(handle, available_len, accessor, mapping)
    }

    pub fn layout_stride(
        self,
        strides: impl Into<Vec<usize>>,
    ) -> Result<MdSpan<'a, T, LayoutStrideMapping<E>, A>, MdSpanError> {
        let Self {
            handle,
            available_len,
            extents,
            accessor,
            ..
        } = self;
        let mapping = LayoutStrideMapping::new(extents, strides)?;
        Self::finish_parts(handle, available_len, accessor, mapping)
    }

    fn finish_parts<M>(
        handle: A::Pointer,
        available_len: Option<usize>,
        accessor: A,
        mapping: M,
    ) -> Result<MdSpan<'a, T, M, A>, MdSpanError>
    where
        M: Mapping<Extents = E>,
    {
        if let Some(available) = available_len {
            let required = mapping.required_span_size();
            if required > available {
                return Err(MdSpanError::InsufficientElements { required, available });
            }
        }

        unsafe { Ok(MdSpan::from_raw_parts(handle, mapping, accessor)) }
    }
}

impl<'a, T, E> MdSpanMutBuilder<'a, T, E, DefaultAccessor<T>>
where
    E: ExtentsType,
{
    pub fn new(slice: &'a mut [T], extents: E) -> Self {
        let available_len = slice.len();
        Self {
            handle: NonNull::from(&mut *slice).cast::<T>(),
            available_len,
            extents,
            accessor: DefaultAccessor::default(),
            _marker: PhantomData,
        }
    }
}

impl<'a, T, E, A> MdSpanMutBuilder<'a, T, E, A>
where
    E: ExtentsType,
    A: AccessorMut<Element = T>,
{
    pub fn layout_right(self) -> Result<MdSpanMut<'a, T, LayoutRightMapping<E>, A>, MdSpanError> {
        let Self {
            handle,
            available_len,
            extents,
            accessor,
            ..
        } = self;
        let mapping = LayoutRightMapping::new(extents);
        Self::finish_parts(handle, available_len, accessor, mapping)
    }

    pub fn layout_left(self) -> Result<MdSpanMut<'a, T, LayoutLeftMapping<E>, A>, MdSpanError> {
        let Self {
            handle,
            available_len,
            extents,
            accessor,
            ..
        } = self;
        let mapping = LayoutLeftMapping::new(extents);
        Self::finish_parts(handle, available_len, accessor, mapping)
    }

    pub fn layout_stride(
        self,
        strides: impl Into<Vec<usize>>,
    ) -> Result<MdSpanMut<'a, T, LayoutStrideMapping<E>, A>, MdSpanError> {
        let Self {
            handle,
            available_len,
            extents,
            accessor,
            ..
        } = self;
        let mapping = LayoutStrideMapping::new(extents, strides)?;
        Self::finish_parts(handle, available_len, accessor, mapping)
    }

    fn finish_parts<M>(
        handle: A::Pointer,
        available_len: usize,
        accessor: A,
        mapping: M,
    ) -> Result<MdSpanMut<'a, T, M, A>, MdSpanError>
    where
        M: Mapping<Extents = E>,
    {
        let required = mapping.required_span_size();
        if required > available_len {
            return Err(MdSpanError::InsufficientElements {
                required,
                available: available_len,
            });
        }

        unsafe { MdSpanMut::from_raw_parts(handle, mapping, accessor) }
    }
}

impl<'a, T, M, A> MdSpan<'a, T, M, A>
where
    M: Mapping,
    A: Accessor<Element = T>,
{
    pub fn builder<E>(slice: &'a [T], extents: E) -> MdSpanBuilder<'a, T, E, DefaultAccessor<T>>
    where
        E: ExtentsType,
    {
        MdSpanBuilder::new(slice, extents)
    }
    pub fn builder_with_handle<E>(handle: A::Pointer, extents: E, accessor: A) -> MdSpanBuilder<'a, T, E, A>
    where
        E: ExtentsType,
    {
        MdSpanBuilder {
            handle,
            available_len: None,
            extents,
            accessor,
            _marker: PhantomData,
        }
    }

    pub fn constant<E>(
        value: T,
        extents: E,
    ) -> Result<MdSpan<'a, T, LayoutRightMapping<E>, ConstantAccessor<T>>, MdSpanError>
    where
        E: ExtentsType,
        T: Clone + fmt::Debug,
    {
        MdSpanBuilder {
            handle: value,
            available_len: None,
            extents,
            accessor: ConstantAccessor::default(),
            _marker: PhantomData,
        }
        .layout_right()
    }
}

impl<'a, T, M, A> MdSpanMut<'a, T, M, A>
where
    M: Mapping,
    A: AccessorMut<Element = T>,
{
    pub fn builder_mut<E>(slice: &'a mut [T], extents: E) -> MdSpanMutBuilder<'a, T, E, DefaultAccessor<T>>
    where
        E: ExtentsType,
    {
        MdSpanMutBuilder::new(slice, extents)
    }
}
