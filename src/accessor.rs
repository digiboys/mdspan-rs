use core::{fmt, marker::PhantomData, ops::Mul, ptr::NonNull};

pub trait Accessor: Clone + fmt::Debug {
    type Element;
    type Pointer: Clone;
    type Reference<'a>
    where
        Self: 'a;

    fn access<'a>(&self, pointer: &'a Self::Pointer, index: usize) -> Self::Reference<'a>;
    fn offset(&self, pointer: Self::Pointer, index: usize) -> Self::Pointer;
}

pub trait AccessorMut: Accessor {
    type MutReference<'a>
    where
        Self: 'a;

    unsafe fn access_mut<'a>(&self, pointer: &'a mut Self::Pointer, index: usize) -> Self::MutReference<'a>;
}

pub struct DefaultAccessor<T> {
    _marker: PhantomData<T>,
}

impl<T> Clone for DefaultAccessor<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for DefaultAccessor<T> {}

impl<T> Default for DefaultAccessor<T> {
    fn default() -> Self {
        Self { _marker: PhantomData }
    }
}

impl<T> fmt::Debug for DefaultAccessor<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DefaultAccessor")
    }
}

impl<T> Accessor for DefaultAccessor<T> {
    type Element = T;
    type Pointer = NonNull<T>;
    type Reference<'a>
        = &'a T
    where
        Self: 'a;

    fn access<'a>(&self, pointer: &'a Self::Pointer, index: usize) -> Self::Reference<'a> {
        unsafe { &*pointer.as_ptr().add(index) }
    }

    fn offset(&self, pointer: Self::Pointer, index: usize) -> Self::Pointer {
        unsafe { NonNull::new_unchecked(pointer.as_ptr().add(index)) }
    }
}

impl<T> AccessorMut for DefaultAccessor<T> {
    type MutReference<'a>
        = &'a mut T
    where
        Self: 'a;

    unsafe fn access_mut<'a>(&self, pointer: &'a mut Self::Pointer, index: usize) -> Self::MutReference<'a> {
        unsafe { &mut *pointer.as_ptr().add(index) }
    }
}

pub struct ConstantAccessor<T> {
    _marker: PhantomData<T>,
}

impl<T> Clone for ConstantAccessor<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ConstantAccessor<T> {}

impl<T> fmt::Debug for ConstantAccessor<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ConstantAccessor")
    }
}

impl<T> Default for ConstantAccessor<T> {
    fn default() -> Self {
        Self { _marker: PhantomData }
    }
}

impl<T: Clone + fmt::Debug> Accessor for ConstantAccessor<T> {
    type Element = T;
    type Pointer = T;
    type Reference<'a>
        = T
    where
        Self: 'a;

    fn access<'a>(&self, pointer: &'a Self::Pointer, _index: usize) -> Self::Reference<'a> {
        pointer.clone()
    }

    fn offset(&self, pointer: Self::Pointer, _index: usize) -> Self::Pointer {
        pointer
    }
}

#[derive(Clone, Debug)]
pub struct ScaledAccessor<T> {
    scale: T,
}

impl<T> ScaledAccessor<T> {
    pub fn new(scale: T) -> Self {
        Self { scale }
    }

    pub fn scale(&self) -> &T {
        &self.scale
    }
}

impl<T> Accessor for ScaledAccessor<T>
where
    T: Copy + Mul<Output = T> + fmt::Debug,
{
    type Element = T;
    type Pointer = NonNull<T>;
    type Reference<'a>
        = T
    where
        Self: 'a;

    fn access<'a>(&self, pointer: &'a Self::Pointer, index: usize) -> Self::Reference<'a> {
        unsafe { *pointer.as_ptr().add(index) * self.scale }
    }

    fn offset(&self, pointer: Self::Pointer, index: usize) -> Self::Pointer {
        unsafe { NonNull::new_unchecked(pointer.as_ptr().add(index)) }
    }
}
