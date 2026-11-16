//! Piece kinds and colored pieces.

use std::fmt;

use super::color::Color;

/// The six kinds of chess piece, ignoring color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceKind {
    /// Pawn.
    Pawn,
    /// Knight.
    Knight,
    /// Bishop.
    Bishop,
    /// Rook.
    Rook,
    /// Queen.
    Queen,
    /// King.
    King,
}

impl PieceKind {
    /// Number of piece kinds.
    pub const COUNT: usize = 6;
    /// All kinds, in index order.
    pub const ALL: [PieceKind; 6] = [
        PieceKind::Pawn,
        PieceKind::Knight,
        PieceKind::Bishop,
        PieceKind::Rook,
        PieceKind::Queen,
        PieceKind::King,
    ];

    /// Index into per-kind arrays (Pawn = 0 .. King = 5).
    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Inverse of [`PieceKind::index`].
    pub const fn from_index(index: usize) -> Option<PieceKind> {
        match index {
            0 => Some(PieceKind::Pawn),
            1 => Some(PieceKind::Knight),
            2 => Some(PieceKind::Bishop),
            3 => Some(PieceKind::Rook),
            4 => Some(PieceKind::Queen),
            5 => Some(PieceKind::King),
            _ => None,
        }
    }

    /// Lowercase FEN letter: `p n b r q k`.
    pub const fn as_char(self) -> char {
        match self {
            PieceKind::Pawn => 'p',
            PieceKind::Knight => 'n',
            PieceKind::Bishop => 'b',
            PieceKind::Rook => 'r',
            PieceKind::Queen => 'q',
            PieceKind::King => 'k',
        }
    }

    /// Parse a lowercase FEN letter.
    pub const fn from_char(c: char) -> Option<PieceKind> {
        match c {
            'p' => Some(PieceKind::Pawn),
            'n' => Some(PieceKind::Knight),
            'b' => Some(PieceKind::Bishop),
            'r' => Some(PieceKind::Rook),
            'q' => Some(PieceKind::Queen),
            'k' => Some(PieceKind::King),
            _ => None,
        }
    }
}

impl fmt::Display for PieceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

/// A piece with a color, e.g. a white knight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Piece {
    /// Side owning the piece.
    pub color: Color,
    /// Kind of piece.
    pub kind: PieceKind,
}

impl Piece {
    /// Number of distinct colored pieces.
    pub const COUNT: usize = 12;

    /// Combine a color and a kind.
    pub const fn new(color: Color, kind: PieceKind) -> Piece {
        Piece { color, kind }
    }

    /// Index into per-piece arrays: `color * 6 + kind`.
    #[inline]
    pub const fn index(self) -> usize {
        self.color.index() * PieceKind::COUNT + self.kind.index()
    }

    /// Inverse of [`Piece::index`].
    pub const fn from_index(index: usize) -> Option<Piece> {
        if index >= Self::COUNT {
            return None;
        }
        let color = if index < PieceKind::COUNT {
            Color::White
        } else {
            Color::Black
        };
        match PieceKind::from_index(index % PieceKind::COUNT) {
            Some(kind) => Some(Piece::new(color, kind)),
            None => None,
        }
    }

    /// FEN letter: uppercase for white, lowercase for black.
    pub fn as_char(self) -> char {
        let c = self.kind.as_char();
        match self.color {
            Color::White => c.to_ascii_uppercase(),
            Color::Black => c,
        }
    }

    /// Parse a FEN letter; case determines the color.
    pub fn from_char(c: char) -> Option<Piece> {
        let color = if c.is_ascii_uppercase() {
            Color::White
        } else {
            Color::Black
        };
        let kind = PieceKind::from_char(c.to_ascii_lowercase())?;
        Some(Piece::new(color, kind))
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn char_round_trip_all_twelve() {
        for color in Color::ALL {
            for kind in PieceKind::ALL {
                let p = Piece::new(color, kind);
                assert_eq!(Piece::from_char(p.as_char()), Some(p));
            }
        }
        assert_eq!(Piece::from_char('x'), None);
        assert_eq!(Piece::from_char('1'), None);
    }

    #[test]
    fn case_encodes_color() {
        assert_eq!(
            Piece::from_char('N'),
            Some(Piece::new(Color::White, PieceKind::Knight))
        );
        assert_eq!(
            Piece::from_char('n'),
            Some(Piece::new(Color::Black, PieceKind::Knight))
        );
    }

    #[test]
    fn indices_are_unique_and_invertible() {
        let mut seen = [false; Piece::COUNT];
        for color in Color::ALL {
            for kind in PieceKind::ALL {
                let p = Piece::new(color, kind);
                assert!(!seen[p.index()]);
                seen[p.index()] = true;
                assert_eq!(Piece::from_index(p.index()), Some(p));
            }
        }
        assert_eq!(Piece::from_index(12), None);
    }
}
