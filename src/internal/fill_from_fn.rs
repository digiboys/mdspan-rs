pub fn fill_from_fn<'a, T: 'a>(iter: impl Iterator<Item = &'a mut T>, mut f: impl FnMut() -> T) {
    for slot in iter {
        *slot = f();
    }
}
