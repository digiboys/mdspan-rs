use core::mem::{self, MaybeUninit};

pub fn array_from_fn_rev<T, const N: usize, F>(mut f: F) -> [T; N]
where
    F: FnMut(usize) -> T,
{
    let mut array = [const { MaybeUninit::uninit() }; N];
    let mut guard = RevGuard {
        array_mut: &mut array,
        initialized_from: N,
    };

    while guard.initialized_from > 0 {
        let i = guard.initialized_from - 1;
        let item = f(i);

        // SAFETY: `i < N`, each slot is written once, and we move the boundary left by one.
        unsafe {
            guard.push_unchecked(item);
        }
    }

    mem::forget(guard);

    // SAFETY: every element was initialized.
    unsafe { MaybeUninit::array_assume_init(array) }
}

struct RevGuard<'a, T, const N: usize> {
    array_mut: &'a mut [MaybeUninit<T>; N],
    /// Initialized elements are in `initialized_from..N`.
    initialized_from: usize,
}

impl<T, const N: usize> RevGuard<'_, T, N> {
    /// Writes the next element into the initialized suffix.
    ///
    /// # Safety
    /// Must not be called more than `N` times.
    unsafe fn push_unchecked(&mut self, item: T) {
        let next = self.initialized_from - 1;
        unsafe {
            self.array_mut.get_unchecked_mut(next).write(item);
        }
        self.initialized_from = next;
    }
}

impl<T, const N: usize> Drop for RevGuard<'_, T, N> {
    fn drop(&mut self) {
        debug_assert!(self.initialized_from <= N);

        // SAFETY: only `initialized_from..N` has been initialized.
        unsafe {
            self.array_mut
                .get_unchecked_mut(self.initialized_from..N)
                .assume_init_drop();
        }
    }
}
