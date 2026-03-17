use either::Either;

pub trait RevIfExt: DoubleEndedIterator + Sized {
    fn rev_if<D: direction::IsForward>(self) -> Either<Self, std::iter::Rev<Self>> {
        if D::VALUE {
            Either::Left(self)
        } else {
            Either::Right(self.rev())
        }
    }
}

impl<I: DoubleEndedIterator> RevIfExt for I {}
