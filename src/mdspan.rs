use std::marker::PhantomData;

trait Accessor {
    type DataHandle;
}

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
    _marker: PhantomData<&'a TElement>,
}
