pub trait Integer: num_integer::Integer + num_traits::NumAssign + Copy + core::iter::Sum + core::iter::Product {}
impl<T: num_integer::Integer + num_traits::NumAssign + Copy + core::iter::Sum + core::iter::Product> Integer for T {}
