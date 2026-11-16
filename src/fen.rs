//! FEN parsing and serialization.
//!
//! Parsing is strict: six fields, eight full ranks, valid characters,
//! exactly one king per side, at most 16 pieces per side, no pawns on
//! back ranks, and an en passant square consistent with the side to move.
//! Deeper legality checks (e.g. "the side not to move is not in check")
//! need attack detection and arrive with movegen in `v0.0.1a02`.
//!
//! Whitespace between fields is tolerated on input; output is canonical.

use std::fmt;

use crate::board::Board;
use crate::types::{CastlingRights, Color, Piece, PieceKind, Square};

/// Everything that can go wrong while reading a FEN string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FenError {
    /// Expected six space-separated fields.
    FieldCount(usize),
    /// The placement field must contain eight ranks separated by `/`.
    RankCount(usize),
    /// A rank describes the wrong number of squares (board rank, 1..8).
    RankWidth {
        /// Board rank number, 1..8.
        rank: usize,
        /// Squares the rank actually described.
        width: usize,
    },
    /// Not a piece letter and not a digit 1..8.
    InvalidPieceChar(char),
    /// Side to move must be `w` or `b`.
    InvalidSideToMove(String),
    /// Castling must be `-` or unique letters from `KQkq`.
    InvalidCastling(String),
    /// En passant must be `-` or a square like `e3` / `e6`.
    InvalidEnPassant(String),
    /// En passant square is on the wrong rank for the side to move.
    EnPassantRank {
        /// The offending square.
        square: Square,
        /// Side to move.
        side: Color,
    },
    /// Halfmove clock is not a non-negative number.
    InvalidHalfmoveClock(String),
    /// Fullmove number is not a number >= 1.
    InvalidFullmoveNumber(String),
    /// Each side needs exactly one king.
    KingCount {
        /// The offending side.
        color: Color,
        /// Kings found.
        count: u32,
    },
    /// A side may not have more than 16 pieces.
    TooManyPieces {
        /// The offending side.
        color: Color,
        /// Pieces found.
        count: u32,
    },
    /// Pawns cannot stand on rank 1 or rank 8.
    PawnOnBackRank(Square),
}

impl fmt::Display for FenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FenError::FieldCount(n) => write!(f, "expected 6 FEN fields, found {n}"),
            FenError::RankCount(n) => {
                write!(f, "expected 8 ranks in placement field, found {n}")
            }
            FenError::RankWidth { rank, width } => {
                write!(f, "rank {rank} describes {width} squares, expected 8")
            }
            FenError::InvalidPieceChar(c) => {
                write!(f, "invalid character {c:?} in placement field")
            }
            FenError::InvalidSideToMove(s) => {
                write!(f, "invalid side to move {s:?}, expected 'w' or 'b'")
            }
            FenError::InvalidCastling(s) => {
                write!(
                    f,
                    "invalid castling field {s:?}, expected '-' or unique letters from 'KQkq'"
                )
            }
            FenError::InvalidEnPassant(s) => write!(f, "invalid en passant square {s:?}"),
            FenError::EnPassantRank { square, side } => {
                write!(
                    f,
                    "en passant square {square} is impossible with {side} to move"
                )
            }
            FenError::InvalidHalfmoveClock(s) => write!(f, "invalid halfmove clock {s:?}"),
            FenError::InvalidFullmoveNumber(s) => {
                write!(f, "invalid fullmove number {s:?}, expected a number >= 1")
            }
            FenError::KingCount { color, count } => {
                write!(f, "{color} has {count} kings, expected exactly 1")
            }
            FenError::TooManyPieces { color, count } => {
                write!(f, "{color} has {count} pieces, maximum is 16")
            }
            FenError::PawnOnBackRank(sq) => {
                write!(f, "pawn on {sq}: pawns cannot stand on rank 1 or 8")
            }
        }
    }
}

impl std::error::Error for FenError {}

/// Parse a FEN string into a [`Board`].
pub fn parse(fen: &str) -> Result<Board, FenError> {
    let fields: Vec<&str> = fen.split_whitespace().collect();
    if fields.len() != 6 {
        return Err(FenError::FieldCount(fields.len()));
    }

    let mut board = Board::empty();
    parse_placement(fields[0], &mut board)?;

    board.side_to_move = match fields[1] {
        "w" => Color::White,
        "b" => Color::Black,
        other => return Err(FenError::InvalidSideToMove(other.to_string())),
    };

    board.castling = CastlingRights::from_fen(fields[2])
        .ok_or_else(|| FenError::InvalidCastling(fields[2].to_string()))?;

    board.en_passant = if fields[3] == "-" {
        None
    } else {
        let sq = Square::from_algebraic(fields[3])
            .ok_or_else(|| FenError::InvalidEnPassant(fields[3].to_string()))?;
        // White to move => black just double-pushed => target on rank 6.
        // Black to move => white just double-pushed => target on rank 3.
        let expected_rank = match board.side_to_move {
            Color::White => 5,
            Color::Black => 2,
        };
        if sq.rank() != expected_rank {
            return Err(FenError::EnPassantRank {
                square: sq,
                side: board.side_to_move,
            });
        }
        Some(sq)
    };

    board.halfmove_clock = fields[4]
        .parse::<u16>()
        .map_err(|_| FenError::InvalidHalfmoveClock(fields[4].to_string()))?;
    board.fullmove_number = fields[5]
        .parse::<u16>()
        .map_err(|_| FenError::InvalidFullmoveNumber(fields[5].to_string()))?;
    if board.fullmove_number == 0 {
        return Err(FenError::InvalidFullmoveNumber(fields[5].to_string()));
    }

    validate(&board)?;
    Ok(board)
}

/// Serialize a board to a canonical FEN string.
pub fn write(board: &Board) -> String {
    let mut out = String::with_capacity(96);

    for rank in (0..8u8).rev() {
        let mut empty = 0u8;
        for file in 0..8u8 {
            let sq = Square::from_rank_file(rank, file).expect("rank/file in range");
            match board.piece_at(sq) {
                Some(piece) => {
                    if empty > 0 {
                        out.push(char::from_digit(empty as u32, 10).expect("1..8"));
                        empty = 0;
                    }
                    out.push(piece.as_char());
                }
                None => empty += 1,
            }
        }
        if empty > 0 {
            out.push(char::from_digit(empty as u32, 10).expect("1..8"));
        }
        if rank > 0 {
            out.push('/');
        }
    }

    out.push(' ');
    out.push(board.side_to_move().as_char());
    out.push(' ');
    out.push_str(&board.castling_rights().to_string());
    out.push(' ');
    match board.en_passant() {
        Some(sq) => out.push_str(&sq.to_string()),
        None => out.push('-'),
    }
    out.push(' ');
    out.push_str(&board.halfmove_clock().to_string());
    out.push(' ');
    out.push_str(&board.fullmove_number().to_string());
    out
}

/// Parse the placement field: eight ranks, FEN order starts at rank 8.
fn parse_placement(field: &str, board: &mut Board) -> Result<(), FenError> {
    let ranks: Vec<&str> = field.split('/').collect();
    if ranks.len() != 8 {
        return Err(FenError::RankCount(ranks.len()));
    }

    for (i, rank_str) in ranks.iter().enumerate() {
        let rank = 7 - i as u8; // 0-based board rank
        let mut file = 0u8;
        for c in rank_str.chars() {
            if let Some(piece) = Piece::from_char(c) {
                let sq = Square::from_rank_file(rank, file).ok_or_else(|| FenError::RankWidth {
                    rank: rank as usize + 1,
                    width: file as usize + 1,
                })?;
                board.place(sq, piece);
                file += 1;
            } else if let Some(skip) = c.to_digit(10) {
                if skip == 0 || skip > 8 {
                    return Err(FenError::InvalidPieceChar(c));
                }
                file += skip as u8;
                if file > 8 {
                    return Err(FenError::RankWidth {
                        rank: rank as usize + 1,
                        width: file as usize,
                    });
                }
            } else {
                return Err(FenError::InvalidPieceChar(c));
            }
        }
        if file != 8 {
            return Err(FenError::RankWidth {
                rank: rank as usize + 1,
                width: file as usize,
            });
        }
    }
    Ok(())
}

/// Position-level sanity checks that do not require attack detection.
fn validate(board: &Board) -> Result<(), FenError> {
    for color in Color::ALL {
        let kings = board.pieces(color, PieceKind::King).count();
        if kings != 1 {
            return Err(FenError::KingCount {
                color,
                count: kings,
            });
        }
        let total = board.pieces_of(color).count();
        if total > 16 {
            return Err(FenError::TooManyPieces {
                color,
                count: total,
            });
        }
    }
    let pawns =
        board.pieces(Color::White, PieceKind::Pawn) | board.pieces(Color::Black, PieceKind::Pawn);
    for sq in pawns {
        if sq.rank() == 0 || sq.rank() == 7 {
            return Err(FenError::PawnOnBackRank(sq));
        }
    }
    Ok(())
}
