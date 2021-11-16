mod frac;

use frac::Frac;

use std::{fmt::Display, marker::PhantomData, ops::Deref, ptr::NonNull};

#[derive(Debug)]
// TODO: Custom Debug implementation
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
    /// Creates a new [Frc] by boxing the input and initializing ownership fraction.
    pub fn new(v: T) -> Self {
        Box::new(v).into()
    }

    /// Returns the item behind the [Frc] if all splits of the [Frc] have been merged. Otherwise returns the [Frc].
    ///
    /// # Examples
    /// ```
    /// use frc::Frc;
    ///
    /// let mut first = Frc::new(8);
    /// let mut split = first.split();
    ///
    /// let mut split = match split.try_unwrap() {
    ///     Err(s) => s,
    ///     _ => panic!(),
    /// };
    ///
    /// split.merge(first);
    ///
    /// let inner = split.try_unwrap();
    ///
    /// assert!(inner.is_ok());
    /// assert_eq!(inner.unwrap(), 8);
    /// ```
    pub fn try_unwrap(self) -> Result<T, Self> {
        if self.frac.is_one() {
            Ok(unsafe { self.unwrap_unchecked() })
        } else {
            Err(self)
        }
    }

    /// Unwraps the [Frc] without checking if all fractions have been merged into it.
    ///
    /// # Safety
    /// There may not exist any other [Frc] containing the same data
    ///
    /// # Examples
    /// ```
    /// use frc::Frc;
    ///
    /// let mut f = Frc::new(8);
    /// let mut split = f.split();
    /// let third = split.split();
    /// // let inner = unsafe {split.unwrap_unchecked()}; // This is UB
    /// drop(f);
    /// // let inner = unsafe {split.unwrap_unchecked()}; // This is UB
    /// drop(third);
    ///
    /// let inner = unsafe {split.unwrap_unchecked()}; // Ok, since no other Frc referincing the data exists
    /// ```
    pub unsafe fn unwrap_unchecked(self) -> T {
        *Box::from_raw(self.item.as_ptr())
    }

    /// Creates a [Frc], distributing the ownership of the input between itself and the new [Frc].
    ///
    /// # Examples
    /// ```
    /// use frc::Frc;
    ///
    /// let mut first = Frc::new("Abra cadabra");
    /// let second = first.split();
    /// assert_eq!(*first, *second);
    /// ```
    pub fn split(&mut self) -> Self {
        Self {
            item: self.item,
            frac: self.frac.split().unwrap(),
            phantom: self.phantom,
        }
    }

    pub unsafe fn merge_unchecked(&mut self, other: Self) {
        // Silently fails if
        let _ = self.frac.merge(other.frac);
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
    fn single_threaded() {}
}
