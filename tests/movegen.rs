//! Position-level move generation tests.

use aspet::{Board, Color, MoveFlag, MoveList, PieceKind, Square};

fn has(moves: &MoveList, from: &str, to: &str, flag: MoveFlag) -> bool {
    let from = Square::from_algebraic(from).unwrap();
    let to = Square::from_algebraic(to).unwrap();
    moves
        .iter()
        .any(|m| m.from_square() == from && m.to_square() == to && m.flag() == flag)
}

#[test]
fn startpos_has_20_pseudo_legal_moves() {
    let board = Board::start_position();
    let moves = board.generate_moves();
    assert_eq!(moves.len(), 20);
    assert!(has(&moves, "e2", "e4", MoveFlag::DoublePush));
    assert!(has(&moves, "g1", "f3", MoveFlag::Quiet));
}

#[test]
fn pawn_promotions_generate_all_four_pieces() {
    let board = Board::from_fen("8/P7/8/8/8/8/8/4K2k w - - 0 1").unwrap();
    let moves = board.generate_moves();

    assert!(has(&moves, "a7", "a8", MoveFlag::PromoN));
    assert!(has(&moves, "a7", "a8", MoveFlag::PromoB));
    assert!(has(&moves, "a7", "a8", MoveFlag::PromoR));
    assert!(has(&moves, "a7", "a8", MoveFlag::PromoQ));

    let promo_count = moves
        .iter()
        .filter(|m| m.from_square() == Square::from_algebraic("a7").unwrap())
        .filter(|m| m.to_square() == Square::from_algebraic("a8").unwrap())
        .count();
    assert_eq!(promo_count, 4);

    let queen_promo = moves.iter().find(|m| m.flag() == MoveFlag::PromoQ).unwrap();
    assert_eq!(queen_promo.promotion_kind(), Some(PieceKind::Queen));
}

#[test]
fn en_passant_is_generated() {
    let board =
        Board::from_fen("rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3").unwrap();
    let moves = board.generate_moves();
    assert!(has(&moves, "e5", "d6", MoveFlag::EpCapture));
}

#[test]
fn castling_is_generated_when_safe() {
    let board = Board::from_fen("4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1").unwrap();
    let moves = board.generate_moves();
    assert!(has(&moves, "e1", "g1", MoveFlag::KingCastle));
    assert!(has(&moves, "e1", "c1", MoveFlag::QueenCastle));
}

#[test]
fn cannot_castle_through_attacked_square() {
    // Black rook on f8 attacks f1, so white kingside castling is disallowed.
    let board = Board::from_fen("4kr2/8/8/8/8/8/8/R3K2R w KQ - 0 1").unwrap();
    let moves = board.generate_moves();
    assert!(!has(&moves, "e1", "g1", MoveFlag::KingCastle));
    assert!(has(&moves, "e1", "c1", MoveFlag::QueenCastle));
}

#[test]
fn check_detection() {
    let board = Board::from_fen("4k3/8/8/8/8/8/4r3/4K3 w - - 0 1").unwrap();
    assert!(board.is_in_check(Color::White));
    assert!(!board.is_in_check(Color::Black));
}
