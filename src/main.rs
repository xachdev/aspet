//! Command-line entry point. The UCI protocol arrives in `v0.0.1a04`.

use std::env;
use std::process::ExitCode;
use std::time::Instant;

use aspet::board::STARTPOS_FEN;
use aspet::perft::{divide, perft};
use aspet::{version, Board};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--version" | "-V") => {
            println!("{}", version::full_name());
            ExitCode::SUCCESS
        }
        Some("--help" | "-h") => {
            print_help();
            ExitCode::SUCCESS
        }
        Some("fen") => show(&args[1..].join(" ")),
        Some("moves") => show_moves(&args[1..].join(" ")),
        Some("perft") => run_perft(&args[1..]),
        Some("divide") => run_divide(&args[1..]),
        Some("makefen") => show_make_fen(&args[1..].join(" ")),
        None => show(STARTPOS_FEN),
        Some(other) => {
            eprintln!("unknown command {other:?}\n");
            print_help();
            ExitCode::FAILURE
        }
    }
}

fn show(fen: &str) -> ExitCode {
    match Board::from_fen(fen) {
        Ok(board) => {
            println!("{}", board.pretty());
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn show_moves(fen: &str) -> ExitCode {
    match Board::from_fen(fen) {
        Ok(board) => {
            let mut board = board;
            let moves = board.generate_legal_moves();
            println!("{} legal moves:", moves.len());
            for mv in moves.iter() {
                println!("{mv}");
            }
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn show_make_fen(fen: &str) -> ExitCode {
    match Board::from_fen(fen) {
        Ok(mut board) => {
            let moves = board.generate_legal_moves();
            for mv in moves.iter().copied() {
                let undo = board.make_move(mv);
                println!("{mv} {}", board.to_fen());
                board.unmake_move(mv, undo);
            }
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn depth_and_board(args: &[String]) -> Result<(u32, Board), String> {
    let depth = args
        .first()
        .ok_or_else(|| "missing depth".to_string())?
        .parse::<u32>()
        .map_err(|_| "depth must be a non-negative integer".to_string())?;
    let fen = if args.len() > 1 {
        args[1..].join(" ")
    } else {
        STARTPOS_FEN.to_string()
    };
    let board = Board::from_fen(&fen).map_err(|e| e.to_string())?;
    Ok((depth, board))
}

fn run_perft(args: &[String]) -> ExitCode {
    let (depth, mut board) = match depth_and_board(args) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    let start = Instant::now();
    let nodes = perft(&mut board, depth);
    let elapsed = start.elapsed().as_secs_f64();
    let nps = if elapsed > 0.0 {
        nodes as f64 / elapsed
    } else {
        0.0
    };
    println!("perft({depth}) = {nodes}");
    println!("time: {elapsed:.3}s  nps: {nps:.0}");
    ExitCode::SUCCESS
}

fn run_divide(args: &[String]) -> ExitCode {
    let (depth, mut board) = match depth_and_board(args) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    let rows = divide(&mut board, depth);
    let mut total = 0u64;
    for (mv, count) in rows {
        println!("{mv}: {count}");
        total += count;
    }
    println!("total: {total}");
    ExitCode::SUCCESS
}

fn print_help() {
    println!("{} — a simple, clear chess engine", version::full_name());
    println!();
    println!("Usage:");
    println!("  aspet                 show the starting position");
    println!("  aspet fen <FEN>       parse and display a FEN position");
    println!("  aspet moves <FEN>     generate legal moves");
    println!("  aspet perft <D> [FEN] count legal sequences to depth D");
    println!("  aspet divide <D> [FEN] perft split by first move");
    println!("  aspet --version       print version");
    println!("  aspet --help          print this help");
}
