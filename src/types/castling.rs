//! Castling rights as a small flag set.

use std::fmt;
use std::ops::{BitOr, BitOrAssign};

/// Which castlings are still allowed: four independent flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct CastlingRights(u8);

impl CastlingRights {
    /// No castling rights (FEN `-`).
    pub const NONE: CastlingRights = CastlingRights(0);
    /// White kingside (FEN `K`).
    pub const WHITE_KINGSIDE: CastlingRights = CastlingRights(0b0001);
    /// White queenside (FEN `Q`).
    pub const WHITE_QUEENSIDE: CastlingRights = CastlingRights(0b0010);
    /// Black kingside (FEN `k`).
    pub const BLACK_KINGSIDE: CastlingRights = CastlingRights(0b0100);
    /// Black queenside (FEN `q`).
    pub const BLACK_QUEENSIDE: CastlingRights = CastlingRights(0b1000);
    /// All four rights (FEN `KQkq`).
    pub const ALL: CastlingRights = CastlingRights(0b1111);

    /// True if no rights remain.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// True if every flag in `flag` is present.
    pub const fn contains(self, flag: CastlingRights) -> bool {
        (self.0 & flag.0) == flag.0
    }

    /// Add rights.
    pub fn insert(&mut self, flag: CastlingRights) {
        self.0 |= flag.0;
    }

    /// Remove rights.
    pub fn remove(&mut self, flag: CastlingRights) {
        self.0 &= !flag.0;
    }

    /// Parse the FEN castling field: `-` or unique letters from `KQkq`.
    ///
    /// Letter order is accepted freely on input; output is canonical.
    pub fn from_fen(s: &str) -> Option<CastlingRights> {
        if s == "-" {
            return Some(CastlingRights::NONE);
        }
        if s.is_empty() {
            return None;
        }
        let mut rights = CastlingRights::NONE;
        for c in s.chars() {
            let flag = match c {
                'K' => Self::WHITE_KINGSIDE,
                'Q' => Self::WHITE_QUEENSIDE,
                'k' => Self::BLACK_KINGSIDE,
                'q' => Self::BLACK_QUEENSIDE,
                _ => return None,
            };
            if rights.contains(flag) {
                return None; // duplicate right
            }
            rights.insert(flag);
        }
        Some(rights)
    }
}

impl BitOr for CastlingRights {
    type Output = CastlingRights;
    fn bitor(self, rhs: CastlingRights) -> CastlingRights {
        CastlingRights(self.0 | rhs.0)
    }
}

impl BitOrAssign for CastlingRights {
    fn bitor_assign(&mut self, rhs: CastlingRights) {
        self.0 |= rhs.0;
    }
}

impl fmt::Display for CastlingRights {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return f.write_str("-");
        }
        for (flag, c) in [
            (Self::WHITE_KINGSIDE, 'K'),
            (Self::WHITE_QUEENSIDE, 'Q'),
            (Self::BLACK_KINGSIDE, 'k'),
            (Self::BLACK_QUEENSIDE, 'q'),
        ] {
            if self.contains(flag) {
                write!(f, "{c}")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fen_round_trip() {
        for s in ["-", "K", "Q", "k", "q", "KQ", "kq", "Kq", "KQkq"] {
            let rights = CastlingRights::from_fen(s).unwrap();
            assert_eq!(rights.to_string(), s);
        }
    }

    #[test]
    fn rejects_invalid() {
        assert_eq!(CastlingRights::from_fen(""), None);
        assert_eq!(CastlingRights::from_fen("KQkqK"), None); // duplicate
        assert_eq!(CastlingRights::from_fen("X"), None);
        assert_eq!(CastlingRights::from_fen("-K"), None);
    }

    #[test]
    fn flag_operations() {
        let mut r = CastlingRights::WHITE_KINGSIDE | CastlingRights::BLACK_QUEENSIDE;
        assert!(r.contains(CastlingRights::WHITE_KINGSIDE));
        assert!(!r.contains(CastlingRights::WHITE_QUEENSIDE));
        r.remove(CastlingRights::WHITE_KINGSIDE);
        assert_eq!(r, CastlingRights::BLACK_QUEENSIDE);
        r |= CastlingRights::ALL;
        assert_eq!(r, CastlingRights::ALL);
        assert!(CastlingRights::NONE.is_empty());
    }
}
