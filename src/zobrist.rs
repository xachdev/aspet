//! Deterministic Zobrist hashing.
//!
//! Keys are generated at compile time from a fixed seed using SplitMix64,
//! so they are identical across runs and platforms. No dependencies.

use crate::types::{CastlingRights, Piece, Square};

/// Fixed seed ("ASPETZOB" in ASCII) - keys are deterministic.
const SEED: u64 = 0x4153_5045_545A_4F42;

/// One SplitMix64 step: returns (next state, output value).
const fn splitmix64(state: u64) -> (u64, u64) {
    let s = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = s;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    (s, z ^ (z >> 31))
}

struct Keys {
    piece_square: [u64; 768],
    castling: [u64; 16],
    ep_file: [u64; 8],
    side: u64,
}

const KEYS: Keys = build_keys();

const fn build_keys() -> Keys {
    let mut state = SEED;

    let mut piece_square = [0u64; 768];
    let mut i = 0;
    while i < 768 {
        let (s, k) = splitmix64(state);
        state = s;
        piece_square[i] = k;
        i += 1;
    }

    let mut castling = [0u64; 16];
    let mut i = 0;
    while i < 16 {
        let (s, k) = splitmix64(state);
        state = s;
        castling[i] = k;
        i += 1;
    }

    let mut ep_file = [0u64; 8];
    let mut i = 0;
    while i < 8 {
        let (s, k) = splitmix64(state);
        state = s;
        ep_file[i] = k;
        i += 1;
    }

    let (_, side) = splitmix64(state);
    Keys {
        piece_square,
        castling,
        ep_file,
        side,
    }
}

/// Key component for a piece on a square.
pub fn piece_square(piece: Piece, sq: Square) -> u64 {
    KEYS.piece_square[piece.index() * 64 + sq.index()]
}

/// Key component for a castling-rights combination.
pub fn castling(rights: CastlingRights) -> u64 {
    KEYS.castling[rights.bits() as usize]
}

/// Key component for an en passant file.
pub fn en_passant(file: u8) -> u64 {
    KEYS.ep_file[file as usize]
}

/// Key component XORed in when black is to move.
pub const SIDE: u64 = KEYS.side;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Color, PieceKind};
    use std::collections::HashSet;

    #[test]
    fn piece_square_keys_unique_and_nonzero() {
        let mut seen = HashSet::new();
        for color in Color::ALL {
            for kind in PieceKind::ALL {
                for sq in Square::all() {
                    let k = piece_square(Piece::new(color, kind), sq);
                    assert_ne!(k, 0);
                    assert!(seen.insert(k), "duplicate key");
                }
            }
        }
        assert_eq!(seen.len(), 768);
    }

    #[test]
    fn castling_keys_distinct() {
        let mut seen = HashSet::new();
        for bits in 0..16u8 {
            assert!(seen.insert(castling(CastlingRights::from_bits(bits))));
        }
    }

    #[test]
    fn side_key_nonzero() {
        assert_ne!(SIDE, 0);
    }
}
