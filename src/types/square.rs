//! Board squares: `A1 = 0` through `H8 = 63`.

use std::fmt;

use super::bitboard::Bitboard;

/// A square, indexed `A1 = 0` .. `H8 = 63` (rank-major, little-endian).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Square(u8);

impl Square {
    /// a1, index 0.
    pub const A1: Square = Square(0);
    /// h8, index 63.
    pub const H8: Square = Square(63);
    /// Number of squares on the board.
    pub const COUNT: usize = 64;

    /// Create from a raw index; `None` if out of range.
    pub const fn new(index: u8) -> Option<Square> {
        if index < 64 {
            Some(Square(index))
        } else {
            None
        }
    }

    /// Create from 0-based rank and file; `None` if either is out of range.
    pub const fn from_rank_file(rank: u8, file: u8) -> Option<Square> {
        if rank < 8 && file < 8 {
            Some(Square(rank * 8 + file))
        } else {
            None
        }
    }

    /// Raw index, 0..63.
    #[inline]
    pub const fn index(self) -> usize {
        self.0 as usize
    }

    /// 0-based rank (row); 0 = rank 1.
    #[inline]
    pub const fn rank(self) -> u8 {
        self.0 / 8
    }

    /// 0-based file (column); 0 = file a.
    #[inline]
    pub const fn file(self) -> u8 {
        self.0 % 8
    }

    /// A bitboard containing exactly this square.
    pub const fn bitboard(self) -> Bitboard {
        Bitboard::from_square(self)
    }

    /// Parse algebraic notation like `e4`.
    pub fn from_algebraic(s: &str) -> Option<Square> {
        let b = s.as_bytes();
        if b.len() != 2 {
            return None;
        }
        let file = b[0].wrapping_sub(b'a');
        let rank = b[1].wrapping_sub(b'1');
        Square::from_rank_file(rank, file)
    }

    /// Iterate over all 64 squares in index order.
    pub fn all() -> impl Iterator<Item = Square> {
        (0..64u8).map(Square)
    }

    /// Crate-internal constructor for indices already known to be valid.
    pub(crate) const fn from_index(index: u8) -> Square {
        debug_assert!(index < 64);
        Square(index)
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let file = (b'a' + self.file()) as char;
        let rank = (b'1' + self.rank()) as char;
        write!(f, "{file}{rank}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn algebraic_round_trip_all_64() {
        for sq in Square::all() {
            assert_eq!(Square::from_algebraic(&sq.to_string()), Some(sq));
        }
    }

    #[test]
    fn known_squares() {
        assert_eq!(Square::from_algebraic("a1"), Some(Square::A1));
        assert_eq!(Square::from_algebraic("h8"), Some(Square::H8));
        let e4 = Square::from_algebraic("e4").unwrap();
        assert_eq!(e4.index(), 28);
        assert_eq!(e4.rank(), 3);
        assert_eq!(e4.file(), 4);
    }

    #[test]
    fn rejects_invalid() {
        assert_eq!(Square::from_algebraic("e9"), None);
        assert_eq!(Square::from_algebraic("i4"), None);
        assert_eq!(Square::from_algebraic("e44"), None);
        assert_eq!(Square::from_algebraic(""), None);
        assert_eq!(Square::new(64), None);
        assert_eq!(Square::from_rank_file(8, 0), None);
        assert_eq!(Square::from_rank_file(0, 8), None);
    }
}
