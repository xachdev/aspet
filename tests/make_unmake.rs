//! Make/unmake round-trip and state-restoration tests.

use aspet::board::STARTPOS_FEN;
use aspet::{Board, Color, Move, MoveFlag, Piece, PieceKind, Square};

fn sq(s: &str) -> Square {
    Square::from_algebraic(s).unwrap()
}

fn find_move(board: &mut Board, from: &str, to: &str, flag: MoveFlag) -> Move {
    *board
        .generate_legal_moves()
        .iter()
        .find(|m| m.from_square() == sq(from) && m.to_square() == sq(to) && m.flag() == flag)
        .unwrap_or_else(|| panic!("no move {from}{to} {flag:?}"))
}

/// Every legal move must make/unmake back to the exact original board.
fn round_trip(fen: &str) {
    let mut board = Board::from_fen(fen).unwrap();
    let original = board.clone();
    let moves = board.generate_legal_moves();
    assert!(!moves.is_empty());
    for mv in moves.iter().copied() {
        let undo = board.make_move(mv);
        assert!(board.is_consistent(), "inconsistent after {mv}");
        board.unmake_move(mv, undo);
        assert_eq!(board, original, "not restored after {mv}");
    }
}

#[test]
fn startpos_round_trip() {
    round_trip(STARTPOS_FEN);
}

#[test]
fn kiwipete_round_trip() {
    round_trip("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
}

#[test]
fn tactics_round_trip() {
    round_trip("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
}

#[test]
fn en_passant_capture() {
    let fen = "rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3";
    let mut board = Board::from_fen(fen).unwrap();
    let mv = find_move(&mut board, "e5", "d6", MoveFlag::EpCapture);

    let undo = board.make_move(mv);
    assert_eq!(
        board.piece_at(sq("d6")),
        Some(Piece::new(Color::White, PieceKind::Pawn))
    );
    assert_eq!(board.piece_at(sq("d5")), None);
    assert_eq!(board.piece_at(sq("e5")), None);
    assert_eq!(board.en_passant(), None);
    assert_eq!(board.halfmove_clock(), 0);
    assert!(board.is_consistent());

    board.unmake_move(mv, undo);
    assert_eq!(board.to_fen(), fen);
}

#[test]
fn promotion() {
    let mut board = Board::from_fen("8/P7/8/8/8/8/8/4K2k w - - 0 1").unwrap();
    let mv = find_move(&mut board, "a7", "a8", MoveFlag::PromoQ);

    let undo = board.make_move(mv);
    assert_eq!(
        board.piece_at(sq("a8")),
        Some(Piece::new(Color::White, PieceKind::Queen))
    );
    assert_eq!(board.piece_at(sq("a7")), None);

    board.unmake_move(mv, undo);
    assert_eq!(
        board.piece_at(sq("a7")),
        Some(Piece::new(Color::White, PieceKind::Pawn))
    );
    assert_eq!(board.piece_at(sq("a8")), None);
}

#[test]
fn castling_moves_rook_and_clears_rights() {
    let mut board = Board::from_fen("4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1").unwrap();
    let mv = find_move(&mut board, "e1", "g1", MoveFlag::KingCastle);

    let undo = board.make_move(mv);
    assert_eq!(
        board.piece_at(sq("g1")),
        Some(Piece::new(Color::White, PieceKind::King))
    );
    assert_eq!(
        board.piece_at(sq("f1")),
        Some(Piece::new(Color::White, PieceKind::Rook))
    );
    assert_eq!(board.piece_at(sq("h1")), None);
    assert_eq!(board.piece_at(sq("e1")), None);
    assert!(board.castling_rights().is_empty());

    board.unmake_move(mv, undo);
    assert_eq!(
        board.piece_at(sq("e1")),
        Some(Piece::new(Color::White, PieceKind::King))
    );
    assert_eq!(
        board.piece_at(sq("h1")),
        Some(Piece::new(Color::White, PieceKind::Rook))
    );
    assert_eq!(board.castling_rights().to_string(), "KQ");
}

#[test]
fn repetition_detection() {
    let mut board = Board::start_position();
    let key = board.key();
    for (from, to) in [("g1", "f3"), ("g8", "f6"), ("f3", "g1"), ("f6", "g8")] {
        let mv = find_move(&mut board, from, to, MoveFlag::Quiet);
        let _ = board.make_move(mv); // play forward, no unmake
    }
    assert_eq!(board.key(), key);
    assert!(board.is_repetition());
}
