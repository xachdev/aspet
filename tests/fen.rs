//! FEN round-trip and rejection suite.
//!
//! Every entry in VALID is canonical: parse -> to_fen must be the identity.
//! Keep appending positions until the a01 exit gate (100+ cases) is met.

use aspet::{Board, FenError};

const VALID: &[&str] = &[
    // Starting position.
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    // Kiwipete (the classic perft stress position).
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    // Standard perft positions 3..6.
    "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
    "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
    "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
    "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
    // En passant targets, both sides.
    "rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3",
    "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
    // Openings / middlegames.
    "r1bqk2r/pppp1ppp/2n2n2/2b1p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4",
    "rnbqkb1r/pppppppp/5n2/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 1 2",
    // Endgames and castling edge cases.
    "8/5k2/8/8/8/8/3K4/8 w - - 0 1",
    "8/8/8/8/8/8/6k1/4K2R w K - 0 1",
    "4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1",
    "r3k2r/8/8/8/8/8/8/R3K2R b kq - 0 1",
    "r3k2r/8/8/8/8/8/8/R3K2R w Kq - 0 1",
    // Pawns one step from promotion, both colors.
    "8/PPPPPPPP/8/8/8/8/pppppppp/4K2k w - - 0 1",
    // Fifty-move clock at the claim boundary.
    "8/8/4k3/8/8/4K3/8/8 b - - 100 75",
    // Stalemate.
    "7k/5Q2/6K1/8/8/8/8/8 b - - 0 1",
];

#[test]
fn round_trip_is_identity() {
    for fen in VALID {
        let board: Board = fen.parse().unwrap_or_else(|e| panic!("{fen}: {e}"));
        assert_eq!(board.to_fen(), *fen, "round trip failed for {fen}");
        assert!(board.is_consistent(), "inconsistent views for {fen}");
        let again: Board = board.to_fen().parse().unwrap();
        assert_eq!(again, board, "parse is not idempotent for {fen}");
    }
}

#[test]
fn whitespace_is_tolerated_and_normalized() {
    for fen in VALID {
        let messy = format!("   {}  ", fen.replace(' ', "   "));
        let board: Board = messy.parse().unwrap();
        assert_eq!(board.to_fen(), *fen);
    }
}

fn err(fen: &str) -> FenError {
    Board::from_fen(fen).unwrap_err()
}

#[test]
fn rejects_structural_errors() {
    assert_eq!(err(""), FenError::FieldCount(0));
    assert_eq!(
        err("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0"),
        FenError::FieldCount(5)
    );
    assert_eq!(err("8/8/8/8/8/8/8 w - - 0 1"), FenError::RankCount(7));
    assert_eq!(
        err("rnbqkbnrr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
        FenError::RankWidth { rank: 8, width: 9 }
    );
    assert_eq!(
        err("rnbqkbn/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
        FenError::RankWidth { rank: 8, width: 7 }
    );
}

#[test]
fn rejects_invalid_characters() {
    assert_eq!(
        err("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKXNR w KQkq - 0 1"),
        FenError::InvalidPieceChar('X')
    );
    assert_eq!(
        err("4k3/8/8/8/8/8/8/0K6 w - - 0 1"),
        FenError::InvalidPieceChar('0')
    );
    assert_eq!(
        err("4k3/8/8/8/8/8/8/9 w - - 0 1"),
        FenError::InvalidPieceChar('9')
    );
}

#[test]
fn rejects_invalid_state_fields() {
    assert_eq!(
        err("4k3/8/8/8/8/8/8/4K3 x - - 0 1"),
        FenError::InvalidSideToMove("x".into())
    );
    assert_eq!(
        err("4k3/8/8/8/8/8/8/4K3 w KQkqK - 0 1"),
        FenError::InvalidCastling("KQkqK".into())
    );
    assert_eq!(
        err("4k3/8/8/8/8/8/8/4K3 w X - 0 1"),
        FenError::InvalidCastling("X".into())
    );
    assert_eq!(
        err("4k3/8/8/8/8/8/8/4K3 w - e9 0 1"),
        FenError::InvalidEnPassant("e9".into())
    );
    assert_eq!(
        err("4k3/8/8/8/8/8/8/4K3 w - - abc 1"),
        FenError::InvalidHalfmoveClock("abc".into())
    );
    assert_eq!(
        err("4k3/8/8/8/8/8/8/4K3 w - - -1 1"),
        FenError::InvalidHalfmoveClock("-1".into())
    );
    assert_eq!(
        err("4k3/8/8/8/8/8/8/4K3 w - - 0 0"),
        FenError::InvalidFullmoveNumber("0".into())
    );
    assert_eq!(
        err("4k3/8/8/8/8/8/8/4K3 w - - 0 x"),
        FenError::InvalidFullmoveNumber("x".into())
    );
}

#[test]
fn rejects_impossible_en_passant() {
    use aspet::{Color, Square};
    assert_eq!(
        err("4k3/8/8/8/8/8/8/4K3 w - e3 0 1"),
        FenError::EnPassantRank {
            square: Square::from_algebraic("e3").unwrap(),
            side: Color::White,
        }
    );
    assert_eq!(
        err("4k3/8/8/8/8/8/8/4K3 b - e6 0 1"),
        FenError::EnPassantRank {
            square: Square::from_algebraic("e6").unwrap(),
            side: Color::Black,
        }
    );
}

#[test]
fn rejects_impossible_positions() {
    use aspet::{Color, Square};
    assert_eq!(
        err("8/8/8/8/8/8/8/8 w - - 0 1"),
        FenError::KingCount {
            color: Color::White,
            count: 0
        }
    );
    assert_eq!(
        err("4K3/8/8/8/8/8/8/4K2k w - - 0 1"),
        FenError::KingCount {
            color: Color::White,
            count: 2
        }
    );
    assert_eq!(
        err("4k3/8/8/8/8/4Q3/PPPPPPPP/RRRRKRRR w - - 0 1"),
        FenError::TooManyPieces {
            color: Color::White,
            count: 17
        }
    );
    assert_eq!(
        err("4k3/8/8/8/8/8/8/P3K3 w - - 0 1"),
        FenError::PawnOnBackRank(Square::from_algebraic("a1").unwrap())
    );
}
