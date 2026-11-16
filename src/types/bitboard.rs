//! A 64-bit set of squares.

use std::fmt;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

use super::square::Square;

/// A set of squares, one bit per square (A1 = bit 0).
///
/// Iterating a `Bitboard` yields its squares in ascending index order.
/// `Bitboard` is `Copy`, so `for sq in bb` leaves `bb` usable afterwards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Bitboard(
    /// Raw bits, one per square.
    pub u64,
);

impl Bitboard {
    /// The empty set.
    pub const EMPTY: Bitboard = Bitboard(0);

    /// A set containing exactly one square.
    pub const fn from_square(sq: Square) -> Bitboard {
        Bitboard(1u64 << sq.index())
    }

    /// True if no squares are set.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Number of set squares.
    pub const fn count(self) -> u32 {
        self.0.count_ones()
    }

    /// True if `sq` is in the set.
    pub const fn contains(self, sq: Square) -> bool {
        (self.0 & Self::from_square(sq).0) != 0
    }

    /// Add a square.
    pub fn insert(&mut self, sq: Square) {
        *self |= Self::from_square(sq);
    }

    /// Remove a square.
    pub fn remove(&mut self, sq: Square) {
        *self &= !Self::from_square(sq);
    }

    /// Flip a square.
    pub fn toggle(&mut self, sq: Square) {
        *self ^= Self::from_square(sq);
    }

    /// Least significant set square, if any.
    pub const fn lsb(self) -> Option<Square> {
        if self.is_empty() {
            None
        } else {
            Some(Square::from_index(self.0.trailing_zeros() as u8))
        }
    }

    /// Remove and return the least significant set square.
    pub fn pop_lsb(&mut self) -> Option<Square> {
        let sq = self.lsb()?;
        self.0 &= self.0 - 1;
        Some(sq)
    }
}

impl Iterator for Bitboard {
    type Item = Square;

    fn next(&mut self) -> Option<Square> {
        self.pop_lsb()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.count() as usize;
        (n, Some(n))
    }
}

impl ExactSizeIterator for Bitboard {}

macro_rules! bit_ops {
    ($trait:ident, $method:ident, $assign:ident, $amethod:ident, $op:tt) => {
        impl $trait for Bitboard {
            type Output = Bitboard;
            fn $method(self, rhs: Bitboard) -> Bitboard {
                Bitboard(self.0 $op rhs.0)
            }
        }
        impl $assign for Bitboard {
            fn $amethod(&mut self, rhs: Bitboard) {
                self.0 = self.0 $op rhs.0;
            }
        }
    };
}

bit_ops!(BitAnd, bitand, BitAndAssign, bitand_assign, &);
bit_ops!(BitOr, bitor, BitOrAssign, bitor_assign, |);
bit_ops!(BitXor, bitxor, BitXorAssign, bitxor_assign, ^);

impl Not for Bitboard {
    type Output = Bitboard;
    fn not(self) -> Bitboard {
        Bitboard(!self.0)
    }
}

impl fmt::Display for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in (0..8u8).rev() {
            for file in 0..8u8 {
                let sq = Square::from_index(rank * 8 + file);
                f.write_str(if self.contains(sq) { "X " } else { ". " })?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sq(s: &str) -> Square {
        Square::from_algebraic(s).unwrap()
    }

    #[test]
    fn insert_remove_toggle() {
        let mut bb = Bitboard::EMPTY;
        assert!(bb.is_empty());
        bb.insert(sq("e4"));
        assert!(bb.contains(sq("e4")));
        assert_eq!(bb.count(), 1);
        bb.insert(sq("e4")); // idempotent
        assert_eq!(bb.count(), 1);
        bb.toggle(sq("e4"));
        assert!(bb.is_empty());
        bb.insert(sq("a1"));
        bb.remove(sq("a1"));
        assert!(bb.is_empty());
    }

    #[test]
    fn iteration_is_ascending_and_non_destructive() {
        let bb = sq("a1").bitboard() | sq("e4").bitboard() | sq("h8").bitboard();
        let squares: Vec<Square> = bb.collect();
        assert_eq!(squares, vec![sq("a1"), sq("e4"), sq("h8")]);
        assert_eq!(bb.count(), 3); // Copy: original untouched
    }

    #[test]
    fn pop_lsb_empties() {
        let mut bb = sq("a1").bitboard() | sq("h8").bitboard();
        assert_eq!(bb.pop_lsb(), Some(sq("a1")));
        assert_eq!(bb.pop_lsb(), Some(sq("h8")));
        assert_eq!(bb.pop_lsb(), None);
        assert_eq!(Bitboard::EMPTY.lsb(), None);
    }

    #[test]
    fn set_algebra() {
        let a = sq("a1").bitboard() | sq("e4").bitboard();
        let b = sq("e4").bitboard() | sq("h8").bitboard();
        assert_eq!(a & b, sq("e4").bitboard());
        assert_eq!(
            a | b,
            sq("a1").bitboard() | sq("e4").bitboard() | sq("h8").bitboard()
        );
        assert_eq!(a ^ b, sq("a1").bitboard() | sq("h8").bitboard());
        assert_eq!(!Bitboard::EMPTY, Bitboard(u64::MAX));
    }
}
