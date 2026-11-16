//! The board: piece placement plus game state.

use std::fmt;
use std::str::FromStr;

use crate::fen::{self, FenError};
use crate::types::{Bitboard, CastlingRights, Color, Piece, PieceKind, Square};

/// FEN of the standard starting position.
pub const STARTPOS_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// A chess position.
///
/// Hybrid representation:
/// - 12 piece bitboards + 2 color occupancies for fast set operations,
/// - a 64-entry mailbox for O(1) "what is on this square?".
///
/// Invariant: both views always agree — see [`Board::is_consistent`].
/// Boards can only be constructed through validated FEN parsing, so
/// every `Board` has exactly one king per side and no pawns on back ranks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    pub(crate) pieces: [Bitboard; Piece::COUNT],
    pub(crate) occupied: [Bitboard; Color::COUNT],
    pub(crate) mailbox: [Option<Piece>; Square::COUNT],
    pub(crate) side_to_move: Color,
    pub(crate) castling: CastlingRights,
    pub(crate) en_passant: Option<Square>,
    pub(crate) halfmove_clock: u16,
    pub(crate) fullmove_number: u16,
}

impl Board {
    /// The standard starting position.
    pub fn start_position() -> Board {
        STARTPOS_FEN.parse().expect("STARTPOS_FEN is valid")
    }

    /// Parse a FEN string.
    pub fn from_fen(fen: &str) -> Result<Board, FenError> {
        fen::parse(fen)
    }

    /// Serialize to a canonical FEN string.
    pub fn to_fen(&self) -> String {
        fen::write(self)
    }

    /// The piece on `sq`, if any.
    pub fn piece_at(&self, sq: Square) -> Option<Piece> {
        self.mailbox[sq.index()]
    }

    /// All pieces of one color.
    pub fn pieces_of(&self, color: Color) -> Bitboard {
        self.occupied[color.index()]
    }

    /// All pieces of one color and kind.
    pub fn pieces(&self, color: Color, kind: PieceKind) -> Bitboard {
        self.pieces[Piece::new(color, kind).index()]
    }

    /// All occupied squares.
    pub fn occupied(&self) -> Bitboard {
        self.occupied[0] | self.occupied[1]
    }

    /// Side to move.
    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    /// Castling rights.
    pub fn castling_rights(&self) -> CastlingRights {
        self.castling
    }

    /// En passant target square, if any.
    pub fn en_passant(&self) -> Option<Square> {
        self.en_passant
    }

    /// Halfmove clock for the fifty-move rule.
    pub fn halfmove_clock(&self) -> u16 {
        self.halfmove_clock
    }

    /// Fullmove number; starts at 1.
    pub fn fullmove_number(&self) -> u16 {
        self.fullmove_number
    }

    /// King square of `color`. Every validated board has exactly one.
    pub fn king_square(&self, color: Color) -> Square {
        self.pieces(color, PieceKind::King)
            .lsb()
            .expect("validated board: exactly one king per side")
    }

    /// Check that the bitboards and the mailbox agree.
    /// Used by tests now, and by fuzz targets in `v0.0.1b01`.
    pub fn is_consistent(&self) -> bool {
        let mut rebuilt = [Bitboard::EMPTY; Piece::COUNT];
        let mut occ = [Bitboard::EMPTY; Color::COUNT];
        for sq in Square::all() {
            if let Some(p) = self.mailbox[sq.index()] {
                rebuilt[p.index()].insert(sq);
                occ[p.color.index()].insert(sq);
            }
        }
        rebuilt == self.pieces && occ == self.occupied
    }

    /// Human-readable multi-line view. `Display` itself writes the FEN.
    pub fn pretty(&self) -> Pretty<'_> {
        Pretty(self)
    }

    /// An empty board, used only as scratch space by the FEN parser.
    pub(crate) fn empty() -> Board {
        Board {
            pieces: [Bitboard::EMPTY; Piece::COUNT],
            occupied: [Bitboard::EMPTY; Color::COUNT],
            mailbox: [None; Square::COUNT],
            side_to_move: Color::White,
            castling: CastlingRights::NONE,
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    /// Place a piece on an empty square, keeping both views in sync.
    pub(crate) fn place(&mut self, sq: Square, piece: Piece) {
        debug_assert!(self.mailbox[sq.index()].is_none());
        self.mailbox[sq.index()] = Some(piece);
        self.pieces[piece.index()].insert(sq);
        self.occupied[piece.color.index()].insert(sq);
    }
}

impl FromStr for Board {
    type Err = FenError;
    fn from_str(s: &str) -> Result<Board, FenError> {
        fen::parse(s)
    }
}

impl fmt::Display for Board {
    /// Writes the canonical FEN, so `fen.parse::<Board>()?.to_string()`
    /// round-trips.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_fen())
    }
}

/// Human-readable board view, created by [`Board::pretty`].
pub struct Pretty<'a>(&'a Board);

impl fmt::Display for Pretty<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let b = self.0;
        writeln!(f, "  +-----------------+")?;
        for rank in (0..8u8).rev() {
            write!(f, "{} | ", rank + 1)?;
            for file in 0..8u8 {
                let sq = Square::from_rank_file(rank, file).expect("rank/file in range");
                match b.piece_at(sq) {
                    Some(p) => write!(f, "{p} ")?,
                    None => f.write_str(". ")?,
                }
            }
            writeln!(f, "|")?;
        }
        writeln!(f, "  +-----------------+")?;
        writeln!(f, "    a b c d e f g h")?;
        writeln!(f, "FEN: {b}")?;
        writeln!(f, "Side to move: {}", b.side_to_move())?;
        writeln!(f, "Castling: {}", b.castling_rights())?;
        match b.en_passant() {
            Some(sq) => writeln!(f, "En passant: {sq}")?,
            None => writeln!(f, "En passant: -")?,
        }
        writeln!(f, "Halfmove clock: {}", b.halfmove_clock())?;
        write!(f, "Fullmove number: {}", b.fullmove_number())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sq(s: &str) -> Square {
        Square::from_algebraic(s).unwrap()
    }

    #[test]
    fn start_position_material() {
        let b = Board::start_position();
        for c in Color::ALL {
            assert_eq!(b.pieces_of(c).count(), 16);
            assert_eq!(b.pieces(c, PieceKind::Pawn).count(), 8);
            assert_eq!(b.pieces(c, PieceKind::King).count(), 1);
        }
        assert_eq!(b.occupied().count(), 32);
        assert!(b.is_consistent());
    }

    #[test]
    fn start_position_pieces() {
        let b = Board::start_position();
        assert_eq!(
            b.piece_at(sq("e1")),
            Some(Piece::new(Color::White, PieceKind::King))
        );
        assert_eq!(
            b.piece_at(sq("d8")),
            Some(Piece::new(Color::Black, PieceKind::Queen))
        );
        assert_eq!(b.piece_at(sq("e4")), None);
        assert_eq!(b.king_square(Color::White), sq("e1"));
        assert_eq!(b.king_square(Color::Black), sq("e8"));
    }

    #[test]
    fn start_position_state() {
        let b = Board::start_position();
        assert_eq!(b.side_to_move(), Color::White);
        assert_eq!(b.castling_rights(), CastlingRights::ALL);
        assert_eq!(b.en_passant(), None);
        assert_eq!(b.halfmove_clock(), 0);
        assert_eq!(b.fullmove_number(), 1);
    }

    #[test]
    fn display_is_fen() {
        let b = Board::start_position();
        assert_eq!(b.to_string(), STARTPOS_FEN);
        assert_eq!(b.to_fen(), STARTPOS_FEN);
    }
}
