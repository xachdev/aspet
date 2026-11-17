//! The board: piece placement plus game state.

use std::fmt;
use std::str::FromStr;

use crate::fen::{self, FenError};
use crate::movegen;
use crate::types::{
    Bitboard, CastlingRights, Color, Move, MoveFlag, MoveList, Piece, PieceKind, Square,
};
use crate::zobrist;

/// FEN of the standard starting position.
pub const STARTPOS_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// Rights retained after a piece moves from/to each square.
/// Moving a king clears both rights; moving a rook or capturing on a
/// corner clears that side's right.
const CASTLE_MASK: [CastlingRights; 64] = build_castle_masks();

const fn build_castle_masks() -> [CastlingRights; 64] {
    let mut m = [CastlingRights::ALL; 64];
    m[0] = CastlingRights::from_bits(0b1101); // a1: white queenside
    m[4] = CastlingRights::from_bits(0b1100); // e1: white both
    m[7] = CastlingRights::from_bits(0b1110); // h1: white kingside
    m[56] = CastlingRights::from_bits(0b0111); // a8: black queenside
    m[60] = CastlingRights::from_bits(0b0011); // e8: black both
    m[63] = CastlingRights::from_bits(0b1011); // h8: black kingside
    m
}

/// Rook source/destination for a castling move.
fn rook_squares(us: Color, flag: MoveFlag) -> (Square, Square) {
    let rank = if us == Color::White { 0u8 } else { 7u8 };
    if flag == MoveFlag::KingCastle {
        (
            Square::from_index(rank * 8 + 7),
            Square::from_index(rank * 8 + 5),
        )
    } else {
        (
            Square::from_index(rank * 8),
            Square::from_index(rank * 8 + 3),
        )
    }
}

/// A chess position.
///
/// Hybrid representation:
/// - 12 piece bitboards + 2 color occupancies for fast set operations,
/// - a 64-entry mailbox for O(1) "what is on this square?",
/// - an incremental Zobrist key and a key history for repetition detection.
///
/// Invariant: the bitboards, mailbox, and key always agree — see
/// [`Board::is_consistent`]. Boards can only be constructed through
/// validated FEN parsing, so every `Board` has exactly one king per side
/// and no pawns on back ranks.
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
    pub(crate) key: u64,
    pub(crate) history: Vec<u64>,
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

    /// Current Zobrist key.
    pub fn key(&self) -> u64 {
        self.key
    }

    /// Number of times the current position occurred earlier in the game.
    pub fn repetitions(&self) -> usize {
        self.history.iter().filter(|&&k| k == self.key).count()
    }

    /// True if the current position occurred at least once before.
    pub fn is_repetition(&self) -> bool {
        self.repetitions() >= 1
    }

    /// True if the current position occurred at least twice before (threefold).
    pub fn is_threefold_repetition(&self) -> bool {
        self.repetitions() >= 2
    }

    /// True if `sq` is attacked by `by`.
    pub fn is_square_attacked(&self, sq: Square, by: Color) -> bool {
        movegen::is_square_attacked(self, sq, by)
    }

    /// True if `color`'s king is in check.
    pub fn is_in_check(&self, color: Color) -> bool {
        movegen::in_check(self, color)
    }

    /// Generate pseudo-legal moves for the side to move.
    ///
    /// These may leave the mover's king in check; use
    /// [`Board::generate_legal_moves`] for search and perft.
    pub fn generate_moves(&self) -> MoveList {
        movegen::generate_moves(self)
    }

    /// Generate legal moves for the side to move.
    pub fn generate_legal_moves(&mut self) -> MoveList {
        movegen::generate_legal_moves(self)
    }

    /// Apply a move, returning the state needed to undo it.
    ///
    /// Works for any pseudo-legal move; legality is checked separately.
    pub fn make_move(&mut self, mv: Move) -> Undo {
        let from = mv.from_square();
        let to = mv.to_square();
        let flag = mv.flag();
        let us = self.side_to_move;
        let piece = self.mailbox[from.index()].expect("move starts on an occupied square");
        debug_assert_eq!(piece.color, us);

        let mut undo = Undo {
            captured: None,
            castling: self.castling,
            en_passant: self.en_passant,
            halfmove_clock: self.halfmove_clock,
            key: self.key,
        };
        self.history.push(self.key);

        // Remove old castling / en passant key components.
        self.key ^= zobrist::castling(self.castling);
        if let Some(ep) = self.en_passant {
            self.key ^= zobrist::en_passant(ep.file());
        }

        // Capture. En passant captures the pawn behind the target square.
        if flag.is_capture() {
            let captured_sq = if flag.is_en_passant() {
                let idx = if us == Color::White {
                    to.index() - 8
                } else {
                    to.index() + 8
                };
                Square::from_index(idx as u8)
            } else {
                to
            };
            let captured = self.remove_piece(captured_sq);
            undo.captured = Some((captured, captured_sq));
        }

        // Move the piece; promotions replace the pawn.
        self.remove_piece(from);
        let placed = match flag.promotion_kind() {
            Some(kind) => Piece::new(us, kind),
            None => piece,
        };
        self.add_piece(to, placed);

        // Castling also moves the rook.
        if flag.is_castling() {
            let (rook_from, rook_to) = rook_squares(us, flag);
            let rook = self.remove_piece(rook_from);
            self.add_piece(rook_to, rook);
        }

        // Castling rights: king/rook moves and captures on corners.
        self.castling &= CASTLE_MASK[from.index()] & CASTLE_MASK[to.index()];
        self.key ^= zobrist::castling(self.castling);

        // En passant target exists only after a double push.
        self.en_passant = if flag == MoveFlag::DoublePush {
            let idx = if us == Color::White {
                from.index() + 8
            } else {
                from.index() - 8
            };
            Some(Square::from_index(idx as u8))
        } else {
            None
        };
        if let Some(ep) = self.en_passant {
            self.key ^= zobrist::en_passant(ep.file());
        }

        // Clocks and side to move.
        self.halfmove_clock = if piece.kind == PieceKind::Pawn || flag.is_capture() {
            0
        } else {
            self.halfmove_clock + 1
        };
        if us == Color::Black {
            self.fullmove_number += 1;
        }
        self.side_to_move = us.opposite();
        self.key ^= zobrist::SIDE;

        undo
    }

    /// Undo a move using the token returned by [`Board::make_move`].
    pub fn unmake_move(&mut self, mv: Move, undo: Undo) {
        let from = mv.from_square();
        let to = mv.to_square();
        let flag = mv.flag();

        self.side_to_move = self.side_to_move.opposite();
        let us = self.side_to_move;
        if us == Color::Black {
            self.fullmove_number -= 1;
        }

        // Move the piece back, undoing any promotion.
        let moved = self.remove_piece(to);
        let original = if flag.is_promotion() {
            Piece::new(us, PieceKind::Pawn)
        } else {
            moved
        };
        self.add_piece(from, original);

        // Castling rook back.
        if flag.is_castling() {
            let (rook_from, rook_to) = rook_squares(us, flag);
            let rook = self.remove_piece(rook_to);
            self.add_piece(rook_from, rook);
        }

        // Restore the captured piece.
        if let Some((captured, sq)) = undo.captured {
            self.add_piece(sq, captured);
        }

        self.castling = undo.castling;
        self.en_passant = undo.en_passant;
        self.halfmove_clock = undo.halfmove_clock;
        self.key = undo.key;
        self.history.pop();
    }

    /// Check that the bitboards, mailbox, and Zobrist key agree.
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
        rebuilt == self.pieces && occ == self.occupied && self.key == self.compute_key()
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
            key: 0,
            history: Vec::new(),
        }
    }

    /// Place a piece on an empty square, keeping both views in sync.
    /// Does not touch the hash; the FEN parser computes it at the end.
    pub(crate) fn place(&mut self, sq: Square, piece: Piece) {
        debug_assert!(self.mailbox[sq.index()].is_none());
        self.mailbox[sq.index()] = Some(piece);
        self.pieces[piece.index()].insert(sq);
        self.occupied[piece.color.index()].insert(sq);
    }

    /// Recompute the Zobrist key from scratch (used by FEN parsing and tests).
    pub(crate) fn compute_key(&self) -> u64 {
        let mut key = 0u64;
        for sq in Square::all() {
            if let Some(p) = self.mailbox[sq.index()] {
                key ^= zobrist::piece_square(p, sq);
            }
        }
        key ^= zobrist::castling(self.castling);
        if let Some(ep) = self.en_passant {
            key ^= zobrist::en_passant(ep.file());
        }
        if self.side_to_move == Color::Black {
            key ^= zobrist::SIDE;
        }
        key
    }

    /// Place a piece and update the hash.
    fn add_piece(&mut self, sq: Square, piece: Piece) {
        self.place(sq, piece);
        self.key ^= zobrist::piece_square(piece, sq);
    }

    /// Remove a piece and update the hash.
    fn remove_piece(&mut self, sq: Square) -> Piece {
        let piece = self.mailbox[sq.index()]
            .take()
            .expect("remove from occupied square");
        self.pieces[piece.index()].remove(sq);
        self.occupied[piece.color.index()].remove(sq);
        self.key ^= zobrist::piece_square(piece, sq);
        piece
    }
}

/// Irreversible state saved by [`Board::make_move`].
///
/// Must be passed back to [`Board::unmake_move`] for the same move.
/// Dropping it without unmaking leaves the board in the moved state.
#[derive(Debug, Clone)]
pub struct Undo {
    captured: Option<(Piece, Square)>,
    castling: CastlingRights,
    en_passant: Option<Square>,
    halfmove_clock: u16,
    key: u64,
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
        assert!(b.is_consistent());
    }

    #[test]
    fn display_is_fen() {
        let b = Board::start_position();
        assert_eq!(b.to_string(), STARTPOS_FEN);
        assert_eq!(b.to_fen(), STARTPOS_FEN);
    }
}
