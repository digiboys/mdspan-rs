use core::{marker::PhantomData, ptr::NonNull};

pub trait Accessor {
    // type Element;
    type DataHandle: Clone;
    type ElementRef<'a>
    where
        Self: 'a;

    fn access<'a>(&self, data_handle: &'a Self::DataHandle, index: usize) -> Self::ElementRef<'a>;
    fn offset(&self, data_handle: Self::DataHandle, index: usize) -> Self::DataHandle;
}

pub trait AccessorMut: Accessor {
    type ElementMut<'a>
    where
        Self: 'a;

    /// # Safety
    /// The caller must guarantee that the data handle, offset by index, points to a
    /// valid initialized value.
    unsafe fn access_mut<'a>(&self, data_handle: &'a mut Self::DataHandle, index: usize) -> Self::ElementMut<'a>;
}

#[derive(Debug, Copy, Clone, Default)]
pub struct DefaultAccessor<T> {
    _marker: PhantomData<T>,
}

impl<T> Accessor for DefaultAccessor<T> {
    // type Element = T;
    type DataHandle = NonNull<T>;
    type ElementRef<'a>
        = &'a T
    where
        Self: 'a;

    fn access<'a>(&self, data_handle: &'a Self::DataHandle, index: usize) -> Self::ElementRef<'a> {
        unsafe { data_handle.add(index).as_ref() }
    }

    fn offset(&self, data_handle: Self::DataHandle, index: usize) -> Self::DataHandle {
        unsafe { data_handle.add(index) }
    }
}

impl<T> AccessorMut for DefaultAccessor<T> {
    type ElementMut<'a>
        = &'a mut T
    where
        Self: 'a;

    unsafe fn access_mut<'a>(&self, data_handle: &'a mut Self::DataHandle, index: usize) -> Self::ElementMut<'a> {
        unsafe { data_handle.add(index).as_mut() }
    }
}
