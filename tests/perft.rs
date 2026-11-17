//! The perft correctness gate.
//!
//! Depth 1–3 run on every `cargo test`. Depth 4–6 are `#[ignore]` and run
//! in release mode via CI (depth 6 nightly). Values are the standard
//! chessprogramming.org perft results.

use aspet::perft::{divide, perft};
use aspet::Board;

const POSITIONS: &[(&str, &str, &[u64])] = &[
    (
        "startpos",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        &[20, 400, 8902, 197281, 4865609, 119060324],
    ),
    (
        "kiwipete",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        // depth 6 = 8031647685 (very long; run manually)
        &[48, 2039, 97862, 4085603, 197281973],
    ),
    (
        "pos3",
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        &[14, 191, 2812, 43238, 674624, 11030083],
    ),
    (
        "pos4",
        "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        &[6, 264, 9467, 422333, 15833292, 706045033],
    ),
    (
        "pos5",
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        &[44, 1486, 62379, 2103487, 89941194],
    ),
    (
        "pos6",
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        &[46, 2079, 89890, 3894594, 164075551],
    ),
];

fn check_depth(depth: u32) {
    for (name, fen, counts) in POSITIONS {
        if counts.len() < depth as usize {
            continue;
        }
        let mut board = Board::from_fen(fen).unwrap();
        let expected = counts[(depth - 1) as usize];
        assert_eq!(perft(&mut board, depth), expected, "{name} depth {depth}");
    }
}

#[test]
fn perft_depth_1_to_3() {
    for depth in 1..=3 {
        check_depth(depth);
    }
}

#[test]
#[ignore]
fn perft_depth_4() {
    check_depth(4);
}

#[test]
#[ignore]
fn perft_depth_5() {
    check_depth(5);
}

#[test]
#[ignore]
fn perft_depth_6() {
    check_depth(6);
}

#[test]
fn divide_startpos_depth_2() {
    let mut board = Board::start_position();
    let rows = divide(&mut board, 2);
    assert_eq!(rows.len(), 20);
    let total: u64 = rows.iter().map(|&(_, c)| c).sum();
    assert_eq!(total, 400);
    for &(_, count) in &rows {
        assert_eq!(count, 20);
    }
}
