//! Pseudo-legal move generation and attack detection.
//!
//! This milestone generates pseudo-legal moves: moves that follow piece
//! movement rules but may leave the mover's king in check. Legal filtering
//! arrives with make/unmake in `v0.0.1a03`.
//!
//! Sliding attacks use simple ray loops. No magic bitboards yet.

use crate::board::Board;
use crate::types::{
    Bitboard, CastlingRights, Color, Move, MoveFlag, MoveList, Piece, PieceKind, Square,
};

const KNIGHT_OFFSETS: [(i8, i8); 8] = [
    (1, 2),
    (2, 1),
    (2, -1),
    (1, -2),
    (-1, -2),
    (-2, -1),
    (-2, 1),
    (-1, 2),
];

const KING_OFFSETS: [(i8, i8); 8] = [
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
];

const WHITE_PAWN_OFFSETS: [(i8, i8); 2] = [(1, -1), (1, 1)];
const BLACK_PAWN_OFFSETS: [(i8, i8); 2] = [(-1, -1), (-1, 1)];

const BISHOP_DIRS: [(i8, i8); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
const ROOK_DIRS: [(i8, i8); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

const fn shifted(sq: Square, dr: i8, df: i8) -> Option<Square> {
    let r = sq.rank() as i8 + dr;
    let f = sq.file() as i8 + df;
    if r < 0 || r > 7 || f < 0 || f > 7 {
        None
    } else {
        Square::from_rank_file(r as u8, f as u8)
    }
}

const fn build_table(offsets: &[(i8, i8)]) -> [Bitboard; 64] {
    let mut table = [Bitboard::EMPTY; 64];
    let mut i = 0;
    while i < 64 {
        let sq = Square::from_index(i as u8);
        let mut bb = Bitboard::EMPTY;
        let mut j = 0;
        while j < offsets.len() {
            let (dr, df) = offsets[j];
            if let Some(target) = shifted(sq, dr, df) {
                bb.0 |= 1u64 << target.index();
            }
            j += 1;
        }
        table[i] = bb;
        i += 1;
    }
    table
}

const KNIGHT_ATTACKS: [Bitboard; 64] = build_table(&KNIGHT_OFFSETS);
const KING_ATTACKS: [Bitboard; 64] = build_table(&KING_OFFSETS);
const PAWN_ATTACKS: [[Bitboard; 64]; 2] = [
    build_table(&WHITE_PAWN_OFFSETS),
    build_table(&BLACK_PAWN_OFFSETS),
];

/// Knight attacks from a square.
pub fn knight_attacks(sq: Square) -> Bitboard {
    KNIGHT_ATTACKS[sq.index()]
}

/// King attacks from a square.
pub fn king_attacks(sq: Square) -> Bitboard {
    KING_ATTACKS[sq.index()]
}

/// Pawn attacks from a square, by color.
pub fn pawn_attacks(color: Color, sq: Square) -> Bitboard {
    PAWN_ATTACKS[color.index()][sq.index()]
}

fn ray_attacks(sq: Square, occupied: Bitboard, dirs: &[(i8, i8)]) -> Bitboard {
    let mut attacks = Bitboard::EMPTY;
    for &(dr, df) in dirs {
        let mut cur = sq;
        while let Some(next) = shifted(cur, dr, df) {
            attacks.insert(next);
            if occupied.contains(next) {
                break;
            }
            cur = next;
        }
    }
    attacks
}

/// Bishop attacks from a square, given occupancy.
pub fn bishop_attacks(sq: Square, occupied: Bitboard) -> Bitboard {
    ray_attacks(sq, occupied, &BISHOP_DIRS)
}

/// Rook attacks from a square, given occupancy.
pub fn rook_attacks(sq: Square, occupied: Bitboard) -> Bitboard {
    ray_attacks(sq, occupied, &ROOK_DIRS)
}

/// Queen attacks from a square, given occupancy.
pub fn queen_attacks(sq: Square, occupied: Bitboard) -> Bitboard {
    bishop_attacks(sq, occupied) | rook_attacks(sq, occupied)
}

/// Attacks from a piece on a square, given occupancy.
///
/// Pawns return attack squares only, not pushes.
pub fn attacks_from(piece: Piece, sq: Square, occupied: Bitboard) -> Bitboard {
    match piece.kind {
        PieceKind::Pawn => pawn_attacks(piece.color, sq),
        PieceKind::Knight => knight_attacks(sq),
        PieceKind::Bishop => bishop_attacks(sq, occupied),
        PieceKind::Rook => rook_attacks(sq, occupied),
        PieceKind::Queen => queen_attacks(sq, occupied),
        PieceKind::King => king_attacks(sq),
    }
}

fn square_attacked(board: &Board, sq: Square, by: Color, occupied: Bitboard) -> bool {
    let pawn_hits = pawn_attacks(by.opposite(), sq) & board.pieces(by, PieceKind::Pawn);
    if !pawn_hits.is_empty() {
        return true;
    }

    let knight_hits = knight_attacks(sq) & board.pieces(by, PieceKind::Knight);
    if !knight_hits.is_empty() {
        return true;
    }

    let king_hits = king_attacks(sq) & board.pieces(by, PieceKind::King);
    if !king_hits.is_empty() {
        return true;
    }

    let bishops_queens = board.pieces(by, PieceKind::Bishop) | board.pieces(by, PieceKind::Queen);
    let bishop_hits = bishop_attacks(sq, occupied) & bishops_queens;
    if !bishop_hits.is_empty() {
        return true;
    }

    let rooks_queens = board.pieces(by, PieceKind::Rook) | board.pieces(by, PieceKind::Queen);
    let rook_hits = rook_attacks(sq, occupied) & rooks_queens;
    !rook_hits.is_empty()
}

/// True if `sq` is attacked by `by`.
pub fn is_square_attacked(board: &Board, sq: Square, by: Color) -> bool {
    square_attacked(board, sq, by, board.occupied())
}

/// True if `color`'s king is in check.
pub fn in_check(board: &Board, color: Color) -> bool {
    is_square_attacked(board, board.king_square(color), color.opposite())
}

/// Generate all pseudo-legal moves for the side to move.
pub fn generate_moves(board: &Board) -> MoveList {
    let mut list = MoveList::new();

    let us = board.side_to_move();
    let them = us.opposite();
    let own = board.pieces_of(us);
    let enemy = board.pieces_of(them);
    let occupied = board.occupied();

    for from in own {
        let piece = board.piece_at(from).expect("occupied square has a piece");
        match piece.kind {
            PieceKind::Pawn => generate_pawn(board, from, us, them, enemy, &mut list),
            PieceKind::King => {
                piece_moves(from, piece, own, enemy, occupied, &mut list);
                generate_castling(board, us, them, occupied, &mut list);
            }
            _ => piece_moves(from, piece, own, enemy, occupied, &mut list),
        }
    }

    list
}

fn piece_moves(
    from: Square,
    piece: Piece,
    own: Bitboard,
    enemy: Bitboard,
    occupied: Bitboard,
    list: &mut MoveList,
) {
    let targets = attacks_from(piece, from, occupied) & !own;
    for to in targets {
        let flag = if enemy.contains(to) {
            MoveFlag::Capture
        } else {
            MoveFlag::Quiet
        };
        list.push(Move::new(from, to, flag));
    }
}

fn generate_pawn(
    board: &Board,
    from: Square,
    us: Color,
    them: Color,
    enemy: Bitboard,
    list: &mut MoveList,
) {
    let dir: i8 = if us == Color::White { 1 } else { -1 };
    let start_rank = if us == Color::White { 1 } else { 6 };
    let promo_rank = if us == Color::White { 7 } else { 0 };

    // Single push, and double push from the starting rank.
    if let Some(one) = shifted(from, dir, 0) {
        if board.piece_at(one).is_none() {
            if one.rank() == promo_rank {
                push_promotions(list, from, one, false);
            } else {
                list.push(Move::new(from, one, MoveFlag::Quiet));
            }

            if from.rank() == start_rank {
                if let Some(two) = shifted(from, dir * 2, 0) {
                    if board.piece_at(two).is_none() {
                        list.push(Move::new(from, two, MoveFlag::DoublePush));
                    }
                }
            }
        }
    }

    // Captures, promotion captures, and en passant.
    for to in pawn_attacks(us, from) {
        if enemy.contains(to) {
            if to.rank() == promo_rank {
                push_promotions(list, from, to, true);
            } else {
                list.push(Move::new(from, to, MoveFlag::Capture));
            }
            continue;
        }

        // En passant: the target must be the ep square, empty, and the
        // enemy pawn must sit directly behind it.
        if board.en_passant() != Some(to) || board.piece_at(to).is_some() {
            continue;
        }
        let Some(captured_sq) = shifted(to, -dir, 0) else {
            continue;
        };
        if board.piece_at(captured_sq) == Some(Piece::new(them, PieceKind::Pawn)) {
            list.push(Move::new(from, to, MoveFlag::EpCapture));
        }
    }
}

fn push_promotions(list: &mut MoveList, from: Square, to: Square, capture: bool) {
    let flags = if capture {
        [
            MoveFlag::PromoCaptureN,
            MoveFlag::PromoCaptureB,
            MoveFlag::PromoCaptureR,
            MoveFlag::PromoCaptureQ,
        ]
    } else {
        [
            MoveFlag::PromoN,
            MoveFlag::PromoB,
            MoveFlag::PromoR,
            MoveFlag::PromoQ,
        ]
    };

    for flag in flags {
        list.push(Move::new(from, to, flag));
    }
}

fn generate_castling(
    board: &Board,
    us: Color,
    them: Color,
    occupied: Bitboard,
    list: &mut MoveList,
) {
    let rank = if us == Color::White { 0u8 } else { 7u8 };
    let sq = |file: u8| Square::from_index(rank * 8 + file);
    let king_sq = sq(4);

    if board.piece_at(king_sq) != Some(Piece::new(us, PieceKind::King)) {
        return;
    }
    if in_check(board, us) {
        return;
    }

    // For transit/destination checks, remove the king from occupancy so
    // sliders cannot be incorrectly blocked by the king's current square.
    let occ_without_king = occupied & !king_sq.bitboard();
    let rights = board.castling_rights();

    let empty = |s: Square| board.piece_at(s).is_none();
    let safe = |s: Square| !square_attacked(board, s, them, occ_without_king);
    let rook = |file: u8| board.piece_at(sq(file)) == Some(Piece::new(us, PieceKind::Rook));

    let kingside_flag = if us == Color::White {
        CastlingRights::WHITE_KINGSIDE
    } else {
        CastlingRights::BLACK_KINGSIDE
    };

    let f = sq(5);
    let g = sq(6);
    if rights.contains(kingside_flag) && rook(7) && empty(f) && empty(g) && safe(f) && safe(g) {
        list.push(Move::new(king_sq, g, MoveFlag::KingCastle));
    }

    let queenside_flag = if us == Color::White {
        CastlingRights::WHITE_QUEENSIDE
    } else {
        CastlingRights::BLACK_QUEENSIDE
    };

    let b = sq(1);
    let c = sq(2);
    let d = sq(3);
    if rights.contains(queenside_flag)
        && rook(0)
        && empty(b)
        && empty(c)
        && empty(d)
        && safe(d)
        && safe(c)
    {
        list.push(Move::new(king_sq, c, MoveFlag::QueenCastle));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn brute_leaper(sq: Square, offsets: &[(i8, i8)]) -> Bitboard {
        let mut bb = Bitboard::EMPTY;
        for &(dr, df) in offsets {
            if let Some(target) = shifted(sq, dr, df) {
                bb.insert(target);
            }
        }
        bb
    }

    fn brute_sliding(sq: Square, occupied: Bitboard, dirs: &[(i8, i8)]) -> Bitboard {
        let mut bb = Bitboard::EMPTY;
        for target in Square::all() {
            if target == sq {
                continue;
            }

            let dr = target.rank() as i8 - sq.rank() as i8;
            let df = target.file() as i8 - sq.file() as i8;

            let step = if dr == 0 && df != 0 {
                (0, df.signum())
            } else if df == 0 && dr != 0 {
                (dr.signum(), 0)
            } else if dr != 0 && dr.abs() == df.abs() {
                (dr.signum(), df.signum())
            } else {
                continue;
            };

            if !dirs.contains(&step) {
                continue;
            }

            let mut cur = sq;
            loop {
                cur = shifted(cur, step.0, step.1).expect("target is on ray");
                if cur == target {
                    bb.insert(target);
                    break;
                }
                if occupied.contains(cur) {
                    break;
                }
            }
        }
        bb
    }

    #[test]
    fn leaper_tables_match_brute_force() {
        for sq in Square::all() {
            assert_eq!(knight_attacks(sq), brute_leaper(sq, &KNIGHT_OFFSETS));
            assert_eq!(king_attacks(sq), brute_leaper(sq, &KING_OFFSETS));
            assert_eq!(
                pawn_attacks(Color::White, sq),
                brute_leaper(sq, &WHITE_PAWN_OFFSETS)
            );
            assert_eq!(
                pawn_attacks(Color::Black, sq),
                brute_leaper(sq, &BLACK_PAWN_OFFSETS)
            );
        }
    }

    #[test]
    fn sliding_attacks_match_brute_force() {
        let occupied = Bitboard(0x00ff_00ff_00ff_00ff);
        for sq in Square::all() {
            assert_eq!(
                bishop_attacks(sq, occupied),
                brute_sliding(sq, occupied, &BISHOP_DIRS)
            );
            assert_eq!(
                rook_attacks(sq, occupied),
                brute_sliding(sq, occupied, &ROOK_DIRS)
            );
            assert_eq!(
                queen_attacks(sq, occupied),
                brute_sliding(sq, occupied, &BISHOP_DIRS) | brute_sliding(sq, occupied, &ROOK_DIRS)
            );
        }
    }

    #[test]
    fn startpos_attack_detection() {
        let board = Board::start_position();
        let f3 = Square::from_algebraic("f3").unwrap();
        let e4 = Square::from_algebraic("e4").unwrap();
        let f6 = Square::from_algebraic("f6").unwrap();

        assert!(is_square_attacked(&board, f3, Color::White));
        assert!(!is_square_attacked(&board, e4, Color::White));
        assert!(is_square_attacked(&board, f6, Color::Black));
        assert!(!in_check(&board, Color::White));
        assert!(!in_check(&board, Color::Black));
    }
}
