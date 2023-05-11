use std::cell::UnsafeCell;

pub struct Cell<T> {
    value: UnsafeCell<T>,
}

// Implied by UnsafeCell
// impl<T> !Sync for Cell<T> {}

impl<T> Cell<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
        }
    }

    pub fn set(&self, value: T) {
        // SAFETY: we know no-one else is concurrently mutating self.value (because !Sync)
        unsafe { *self.value.get() = value };
    }

    pub fn get(&self) -> T
    where
        T: Copy,
    {
        unsafe { *self.value.get() }
    }
}

// #[cfg(test)]
// mod test {
//     use std::sync::Arc;
//     use std::thread;

//     use super::Cell;

//     #[test]
//     fn bad() {
//         let x = Arc::new(Cell::new(0));

//         let x1 = Arc::clone(&x);
//         let jh1 = thread::spawn(move || {
//             for _ in 0..1_000_000 {
//                 let v = x1.get();
//                 x1.set(v + 1);
//             }
//         });

//         let x2 = Arc::clone(&x);
//         let jh2 = thread::spawn(move || {
//             for _ in 0..1_000_000 {
//                 let v = x2.get();
//                 x2.set(v + 1);
//             }
//         });

//         jh1.join().unwrap();
//         jh2.join().unwrap();

//         assert_eq!(x.get(), 2_000_000);
//     }
// }
