pub trait Integer:
    num_integer::Integer + num_traits::NumAssign + num_traits::ToPrimitive + Copy + core::iter::Sum + core::iter::Product
{
    fn to_usize(&self) -> usize;
}

impl<
    T: num_integer::Integer
        + num_traits::NumAssign
        + num_traits::ToPrimitive
        + Copy
        + core::iter::Sum
        + core::iter::Product,
> Integer for T
{
    fn to_usize(&self) -> usize {
        num_traits::ToPrimitive::to_usize(self).expect("Integer value could not be converted to usize")
    }
}
