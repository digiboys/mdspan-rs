use std::marker::PhantomData;

use mod_accessor::*;
use mod_layout::*;

pub struct MDRef<'a, TElement, TExtents, TLayout, TAccessor>
where
    TLayout: Layout,
    TAccessor: Accessor,
{
    #[allow(unused)]
    data_handle: TAccessor::DataHandle,
    #[allow(unused)]
    mapping: TLayout::Mapping<TExtents>,
    #[allow(unused)]
    accessor: TAccessor,
    _marker: PhantomData<&'a TElement>,
}
