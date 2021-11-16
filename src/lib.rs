mod frac;

use frac::Frac;

use std::{fmt::Display, marker::PhantomData, ops::Deref, ptr::NonNull};

#[derive(Debug)]
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

    /// Returns the item behind the [Frc].
    ///
    /// # Examples
    ///
    /// ```
    /// use frc::Frc;
    ///
    /// let mut first = Frc::new(8);
    /// let mut split = first.split();
    ///
    /// // let mut split = split.unwrap(); // Would panic!
    ///
    /// split.merge(first);
    ///
    /// let inner = split.unwrap();
    ///
    /// assert_eq!(inner, 8);
    /// ```
    ///
    /// # Panics
    /// If not all splits have been merged into the [Frc]
    pub fn unwrap(self) -> T {
        if let Ok(ret) = self.try_unwrap() {
            ret
        } else {
            panic!("The Frc didn't have complete ownership!")
        }
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
    /// let mut split = split.try_unwrap().unwrap_err();
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
}

impl<T: ?Sized> Frc<T> {
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
    ///
    /// # Panics
    ///
    /// If (2^64) subsplits are made.
    pub fn split(&mut self) -> Self {
        Self {
            item: self.item,
            frac: self.frac.split().unwrap(),
            phantom: self.phantom,
        }
    }

    /// Merges two [Frc]s, adding their partial ownerships together.
    ///
    /// # Examples
    /// ```
    /// use frc::Frc;
    ///
    /// let mut first = Frc::new(vec![1234]);
    /// let mut second = first.split();
    ///
    /// let mut second = second.try_unwrap().unwrap_err();
    ///
    /// second.merge(first);
    ///
    /// assert_eq!(second.unwrap(), vec![1234]);
    /// ```
    ///
    /// # Panics
    /// If an unrelated [Frc] was about to be merged, or if the backing fraction overflows.
    pub fn merge(&mut self, other: Self) {
        assert!(self.item == other.item);
        if let Err(e) = unsafe { self.merge_unchecked(other) } {
            panic!("Failed merging Frcs: {}", e);
        }
    }

    /// Merges two [Frc]s, adding their partial ownerships together. Returns [MergeErr] if the
    ///
    /// # Examples
    /// ```
    /// use frc::Frc;
    ///
    /// let mut first = Frc::new(vec![1234]);
    /// let other = Frc::new(vec![4321]);
    ///
    /// let other = first.try_merge(other)
    ///     .map_err(|e| e.other)
    ///     .unwrap_err();
    ///
    /// let mut second = first.split();
    ///
    /// assert!(second.try_merge(first).is_ok());
    ///
    /// assert_eq!(second.unwrap(), vec![1234]);
    /// ```
    pub fn try_merge(&mut self, other: Self) -> Result<(), MergeErr<T>> {
        if self.item == other.item {
            unsafe { self.merge_unchecked(other) }?;
            Ok(())
        } else {
            Err(MergeErr {
                kind: MergeErrKind::IncompatibleFrcs,
                other,
            })
        }
    }

    pub unsafe fn merge_unchecked(&mut self, other: Self) -> Result<(), MergeErr<T>> {
        self.frac.merge(other.frac).map_err(|_| MergeErr {
            kind: MergeErrKind::FractionOverflow(FracOverflowInfo(self.frac, other.frac)),
            other,
        })?;
        Ok(())
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
pub struct MergeErr<T: ?Sized> {
    pub other: Frc<T>,
    kind: MergeErrKind,
}

#[derive(Debug)]
enum MergeErrKind {
    FractionOverflow(FracOverflowInfo),
    IncompatibleFrcs,
}

#[derive(Debug)]
struct FracOverflowInfo(frac::Frac, frac::Frac);

impl<T: ?Sized> Display for MergeErr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            MergeErrKind::FractionOverflow(FracOverflowInfo(lhs, rhs)) => write!(f, "The inner fractions are too different ({} and {}), and adding them together causes an overflow!", lhs, rhs),
            MergeErrKind::IncompatibleFrcs => write!(f, "Tried merging two Frcs owning different data!"),
        }
    }
}

impl<T: ?Sized + std::fmt::Debug> std::error::Error for MergeErr<T> {}

#[cfg(test)]
mod tests {
    #[test]
    fn single_threaded() {}
}
