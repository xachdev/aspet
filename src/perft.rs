//! Perft: count legal move sequences to a given depth.
//!
//! The correctness oracle for move generation and make/unmake.

use crate::board::Board;
use crate::movegen::generate_legal_moves;
use crate::types::Move;

/// Count legal positions reachable in exactly `depth` plies.
pub fn perft(board: &mut Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let moves = generate_legal_moves(board);
    if depth == 1 {
        return moves.len() as u64; // bulk counting
    }

    let mut nodes = 0u64;
    for mv in moves.iter().copied() {
        let undo = board.make_move(mv);
        nodes += perft(board, depth - 1);
        board.unmake_move(mv, undo);
    }
    nodes
}

/// Perft split by first move, for debugging mismatches.
pub fn divide(board: &mut Board, depth: u32) -> Vec<(Move, u64)> {
    let mut out = Vec::new();
    let moves = generate_legal_moves(board);
    for mv in moves.iter().copied() {
        let undo = board.make_move(mv);
        let count = perft(board, depth.saturating_sub(1));
        board.unmake_move(mv, undo);
        out.push((mv, count));
    }
    out
}

/// Perft with full state-restoration checks at every node. Debugging only.
///
/// Clones the board per node, so it is far slower than [`perft`].
/// Panics with the FEN and the offending move at the first mismatch.
pub fn perft_checked(board: &mut Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let snapshot = board.clone();
    let moves = generate_legal_moves(board);
    assert_eq!(
        *board, snapshot,
        "movegen left the board changed: {}",
        snapshot.to_fen()
    );
    assert!(
        board.is_consistent(),
        "inconsistent after movegen: {}",
        snapshot.to_fen()
    );

    if depth == 1 {
        return moves.len() as u64;
    }

    let mut nodes = 0u64;
    for mv in moves.iter().copied() {
        let undo = board.make_move(mv);
        assert!(
            board.is_consistent(),
            "inconsistent after make {mv}: {}",
            snapshot.to_fen()
        );
        nodes += perft_checked(board, depth - 1);
        board.unmake_move(mv, undo);
        assert_eq!(
            *board, snapshot,
            "unmake mismatch after {mv}: {}",
            snapshot.to_fen()
        );
    }
    nodes
}