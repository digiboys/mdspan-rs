use std::marker::PhantomData;

use mod_accessor::*;

trait Layout {
    type Mapping<TExtents>;
}

struct MDRef<'a, TElement, TExtents, TLayout, TAccessor>
where
    TLayout: Layout,
    TAccessor: Accessor,
{
    data_handle: TAccessor::DataHandle,
    mapping: TLayout::Mapping<TExtents>,
    accessor: TAccessor,
    _marker: PhantomData<&'a TElement>,
}
