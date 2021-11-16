use std::fmt::Display;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Frac {
    num: usize,
    den: usize,
}

impl Frac {
    pub(crate) fn new() -> Self {
        Frac { num: 1, den: 1 }
    }

    pub(crate) fn split(&mut self) -> Result<Self, SplitErr> {
        self.den = self.den.checked_add(1).ok_or(SplitErr())?;
        Ok(Self {
            num: self.num,
            den: self.den,
        })
    }

    pub(crate) fn merge(&mut self, mut other: Self) -> Result<(), MergeErr> {
        let (min_den, max_den) = if self.den < other.den {
            (&mut *self, &mut other)
        } else {
            (&mut other, &mut *self)
        };

        let diff = max_den.den - min_den.den;
        min_den.num = min_den
            .num
            .checked_shl(diff.try_into().map_err(|_| MergeErr())?)
            .ok_or(MergeErr())?;
        let mut num = min_den.num + max_den.num;
        let trailing = num.trailing_zeros() as usize;
        num >>= trailing;
        let den = max_den.den - trailing;

        self.num = num;
        self.den = den;
        Ok(())
    }

    pub(crate) fn is_one(&self) -> bool {
        self.num == 1 && self.den == 1
    }
}

impl std::fmt::Display for Frac {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/((2^{}) - 1)", self.num, self.den)
    }
}

#[derive(Debug)]
pub struct MergeErr();

impl Display for MergeErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Nominator became too big, could not merge!")
    }
}

impl std::error::Error for MergeErr {}

#[derive(Debug)]
pub struct SplitErr();

impl Display for SplitErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Denominater was about to overflow, could not split!")
    }
}
impl std::error::Error for SplitErr {}

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
        let mut half = three_fourth.split().unwrap();
        let quater = half.split().unwrap();
        three_fourth.merge(quater).unwrap();

        assert_eq!(3, three_fourth.num);
        assert_eq!(4, 1 << (three_fourth.den - 1));
    }
}
