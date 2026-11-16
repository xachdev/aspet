# Aspet

*Aspet (ասպետ) — "knight" in Armenian.*

A simple, clear chess engine written in Rust. The goal is code that is easy
to understand, test, and improve — strength follows correctness.

**Status: `v0.0.1a01` — foundation.** Board representation (bitboards +
mailbox), strict FEN parsing/serialization, core types, CI. No move
generation or search yet; see the roadmap.

## Build & run

    cargo run                       # show the starting position
    cargo run -- fen "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1"
    cargo test                      # unit + FEN suite

## Design

- `types/` — Color, Piece, Square, Bitboard, CastlingRights (all `Copy`).
- `board.rs` — 12 piece bitboards + 2 occupancies + 64-entry mailbox.
- `fen.rs` — strict parser with typed errors; canonical writer.
- `#![forbid(unsafe_code)]`, zero dependencies.

## License

GPL-3.0-or-later.