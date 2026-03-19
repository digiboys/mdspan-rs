use integer::Integer;

pub struct ToUsize<I>(I);

impl<I: Iterator> Iterator for ToUsize<I>
where
    I::Item: Integer,
{
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        self.0.next().map(|e| e.to_usize())
    }
}

impl<I: DoubleEndedIterator> DoubleEndedIterator for ToUsize<I>
where
    I::Item: Integer,
{
    fn next_back(&mut self) -> Option<usize> {
        self.0.next_back().map(|e| e.to_usize())
    }
}

impl<I: ExactSizeIterator> ExactSizeIterator for ToUsize<I> where I::Item: Integer {}

pub trait ToUsizeExt: Iterator + Sized {
    fn to_usize(self) -> ToUsize<Self>;
}

impl<I: Iterator> ToUsizeExt for I
where
    I::Item: Integer,
{
    fn to_usize(self) -> ToUsize<Self> {
        ToUsize(self)
    }
}
