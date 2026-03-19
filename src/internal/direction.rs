pub struct Forward;
pub struct Reverse;

pub trait IsForward {
    const VALUE: bool;
}

impl IsForward for Forward {
    const VALUE: bool = true;
}

impl IsForward for Reverse {
    const VALUE: bool = false;
}
