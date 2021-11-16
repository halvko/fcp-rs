use std::{fmt::Display, marker::PhantomData, ops::Deref, ptr::NonNull};

use frac::Frac;

mod frac {
    pub(crate) struct Frac {
        num: usize,
        den: usize,
    }

    impl Frac {
        pub(crate) fn new() -> Self {
            Frac { num: 1, den: 1 }
        }

        pub(crate) fn split(&mut self) -> Self {
            self.den += 1;
            Self {
                num: self.num,
                den: self.den,
            }
        }

        pub(crate) fn merge(&mut self, mut other: Self) {
            let (min_den, max_den) = if self.den < other.den {
                (&mut *self, &mut other)
            } else {
                (&mut other, &mut *self)
            };

            let diff = max_den.den - min_den.den;
            min_den.num <<= diff;
            let mut num = min_den.num + max_den.num;
            let trailing = num.trailing_zeros() as usize;
            num >>= trailing;
            let den = max_den.den - trailing;

            self.num = num;
            self.den = den;
        }

        pub(crate) fn is_one(&self) -> bool {
            self.num == 1 && self.den == 1
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn new_is_one() {
            let frac = Frac::new();
            assert!(frac.is_one());
        }

        #[test]
        fn split_seems_reasonable() {
            let mut three_fourth = Frac::new();
            let mut half = three_fourth.split();
            let quater = half.split();
            three_fourth.merge(quater);

            assert_eq!(3, three_fourth.num);
            assert_eq!(4, 1 << (three_fourth.den - 1));
        }
    }
}

pub struct Frc<T: ?Sized> {
    item: NonNull<T>,
    frac: Frac,
    phantom: PhantomData<T>,
}

unsafe impl<T: Send + Sync> Send for Frc<T> where T: ?Sized {}

impl<T: ?Sized> Frc<T> {
    unsafe fn from_inner(ptr: NonNull<T>) -> Self {
        Self {
            item: ptr,
            frac: Frac::new(),
            phantom: PhantomData,
        }
    }
}

impl<T> Frc<T> {
    pub fn new(v: T) -> Self {
        Box::new(v).into()
    }

    pub fn try_unwrap(self) -> Result<T, Self> {
        if self.frac.is_one() {
            Ok(*unsafe { Box::from_raw(self.item.as_ptr()) })
        } else {
            Err(self)
        }
    }

    pub fn split(&mut self) -> Self {
        Self {
            item: self.item,
            frac: self.frac.split(),
            phantom: self.phantom,
        }
    }

    pub unsafe fn merge_unchecked(&mut self, other: Self) {
        self.frac.merge(other.frac);
    }

    pub fn try_merge(&mut self, other: Self) -> Result<(), MergeErr> {
        if self.item == other.item {
            unsafe { self.merge_unchecked(other) };
            Ok(())
        } else {
            Err(MergeErr())
        }
    }

    pub fn merge(&mut self, other: Self) {
        assert!(self.item == other.item);
        unsafe { self.merge_unchecked(other) };
    }
}

impl<T: ?Sized> From<Box<T>> for Frc<T> {
    fn from(item: Box<T>) -> Self {
        unsafe { Self::from_inner(Box::leak(item).into()) }
    }
}

impl<T: ?Sized> Deref for Frc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.item.as_ref() }
    }
}

#[derive(Debug)]
pub struct MergeErr();

impl Display for MergeErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tried merging Frcs of differing origins")
    }
}

impl std::error::Error for MergeErr {}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
