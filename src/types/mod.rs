//! Core value types shared by every part of the engine.

mod bitboard;
mod castling;
mod color;
mod piece;
mod square;

pub use bitboard::Bitboard;
pub use castling::CastlingRights;
pub use color::Color;
pub use piece::{Piece, PieceKind};
pub use square::Square;
