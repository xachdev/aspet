//! # Aspet
//!
//! A simple, clear chess engine written in Rust.
//!
//! This is `v0.0.1a02`: attack detection and pseudo-legal move generation.
//! Make/unmake, Zobrist hashing, and perft arrive in `v0.0.1a03`.
//!
//! ```
//! use aspet::Board;
//!
//! let board = Board::start_position();
//! let moves = board.generate_moves();
//! assert_eq!(moves.len(), 20);
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod board;
pub mod fen;
pub mod movegen;
pub mod types;
pub mod version;

pub use board::Board;
pub use fen::FenError;
pub use types::{
    Bitboard, CastlingRights, Color, Move, MoveFlag, MoveList, Piece, PieceKind, Square,
};
