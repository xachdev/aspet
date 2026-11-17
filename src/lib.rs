//! # Aspet
//!
//! A simple, clear chess engine written in Rust.
//!
//! This is `v0.0.1a03`: make/unmake, Zobrist hashing, legal move
//! filtering, and perft. UCI arrives in `v0.0.1a04`.
//!
//! ```
//! use aspet::Board;
//!
//! let mut board = Board::start_position();
//! assert_eq!(aspet::perft::perft(&mut board, 3), 8902);
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod board;
pub mod fen;
pub mod movegen;
pub mod perft;
pub mod types;
pub mod version;
pub mod zobrist;

pub use board::{Board, Undo};
pub use fen::FenError;
pub use types::{
    Bitboard, CastlingRights, Color, Move, MoveFlag, MoveList, Piece, PieceKind, Square,
};
