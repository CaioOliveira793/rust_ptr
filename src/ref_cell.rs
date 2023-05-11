use std::cell::UnsafeCell;

#[derive(Clone, Copy)]
enum RefState {
    Unshared,
    Shared(usize),
    Exclusive,
}

// Implied by UnsafeCell
// impl<T> !Sync for Cell<T> {}

pub struct RefCell<T> {
    value: UnsafeCell<T>,
    state: crate::cell::Cell<RefState>,
}

impl<T> RefCell<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
            state: crate::cell::Cell::new(RefState::Unshared),
        }
    }

    pub fn borrow(&self) -> Option<Ref<'_, T>> {
        match self.state.get() {
            RefState::Unshared => {
                self.state.set(RefState::Shared(1));
                Some(Ref::new(self))
            }
            RefState::Shared(n) => {
                self.state.set(RefState::Shared(n + 1));
                Some(Ref::new(self))
            }
            _ => None,
        }
    }

    pub fn borrow_mut(&self) -> Option<RefMut<'_, T>> {
        match self.state.get() {
            RefState::Unshared => {
                self.state.set(RefState::Exclusive);
                Some(RefMut::new(self))
            }
            _ => None,
        }
    }
}

pub struct Ref<'refcell, T> {
    refcell: &'refcell RefCell<T>,
}

impl<'refcell, T> Ref<'refcell, T> {
    fn new(refcell: &'refcell RefCell<T>) -> Self {
        Self { refcell }
    }

    pub fn get(&self) -> &T {
        // SAFETY:
        // a Ref is only created if no exclusive references have been given out.
        // once it is given out, state is set to Shared so no exclusive references are given out.
        // so dereferencing into a shared reference is fine.
        unsafe { &*self.refcell.value.get() }
    }
}

impl<T> std::ops::Deref for Ref<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<'refcell, T> Drop for Ref<'refcell, T> {
    fn drop(&mut self) {
        match self.refcell.state.get() {
            RefState::Unshared | RefState::Exclusive => unreachable!(),
            RefState::Shared(1) => {
                self.refcell.state.set(RefState::Unshared);
            }
            RefState::Shared(n) => {
                self.refcell.state.set(RefState::Shared(n - 1));
            }
        }
    }
}

pub struct RefMut<'refcell, T> {
    refcell: &'refcell RefCell<T>,
}

impl<'refcell, T> RefMut<'refcell, T> {
    fn new(refcell: &'refcell RefCell<T>) -> Self {
        Self { refcell }
    }

    pub fn get(&self) -> &mut T {
        // SAFETY:
        // a RefMut is only created if no other references have been given out.
        // once it is given out, state is set to Exclusive, so no future references are given out.
        // so we have an exclusive lease on the inner value, so mutably dereferencing is fine.
        unsafe { &mut *self.refcell.value.get() }
    }
}

impl<T> std::ops::Deref for RefMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T> std::ops::DerefMut for RefMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get()
    }
}

impl<'refcell, T> Drop for RefMut<'refcell, T> {
    fn drop(&mut self) {
        match self.refcell.state.get() {
            RefState::Unshared | RefState::Shared(_) => unreachable!(),
            RefState::Exclusive => {
                self.refcell.state.set(RefState::Unshared);
            }
        }
    }
}
