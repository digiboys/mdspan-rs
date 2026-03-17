// Do NOT import `NonZero`, that IS the build error.
pub fn trigger_build_error(value: u8) -> Option<NonZero<u8>> {
    NonZero::new(value)
}
