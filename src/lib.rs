//! # Aspet
//!
//! A simple, clear chess engine written in Rust.
//!
//! This is `v0.0.1a01`: the foundation. Core value types, the board
//! representation, and strict FEN parsing/serialization. Move generation,
//! search, and UCI arrive in later alpha builds.
//!
//! ```
//! use aspet::Board;
//!
//! let board = Board::start_position();
//! assert_eq!(board.to_fen(), aspet::board::STARTPOS_FEN);
//! assert!(board.is_consistent());
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod board;
pub mod fen;
pub mod types;
pub mod version;

pub use board::Board;
pub use fen::FenError;
pub use types::{Bitboard, CastlingRights, Color, Piece, PieceKind, Square};
