//! The two sides of a chess game.

use std::fmt;
use std::ops::Not;

/// A side: white or black.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    /// White.
    White,
    /// Black.
    Black,
}

impl Color {
    /// Number of colors.
    pub const COUNT: usize = 2;
    /// Both colors, in index order.
    pub const ALL: [Color; 2] = [Color::White, Color::Black];

    /// Index into per-color arrays (White = 0, Black = 1).
    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// The other side.
    #[inline]
    pub const fn opposite(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    /// FEN character: `w` or `b`.
    pub const fn as_char(self) -> char {
        match self {
            Color::White => 'w',
            Color::Black => 'b',
        }
    }

    /// Parse a FEN side-to-move character.
    pub const fn from_char(c: char) -> Option<Color> {
        match c {
            'w' => Some(Color::White),
            'b' => Some(Color::Black),
            _ => None,
        }
    }
}

impl Not for Color {
    type Output = Color;
    fn not(self) -> Color {
        self.opposite()
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Color::White => "white",
            Color::Black => "black",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opposite_is_involution() {
        for c in Color::ALL {
            assert_eq!(c.opposite().opposite(), c);
            assert_eq!(!c, c.opposite());
            assert_ne!(c, c.opposite());
        }
    }

    #[test]
    fn fen_char_round_trip() {
        for c in Color::ALL {
            assert_eq!(Color::from_char(c.as_char()), Some(c));
        }
        assert_eq!(Color::from_char('x'), None);
    }

    #[test]
    fn indices() {
        assert_eq!(Color::White.index(), 0);
        assert_eq!(Color::Black.index(), 1);
    }
}
