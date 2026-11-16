//! Command-line entry point. The UCI protocol arrives in `v0.0.1a04`.

use std::env;
use std::process::ExitCode;

use aspet::board::STARTPOS_FEN;
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

fn print_help() {
    println!("{} — a simple, clear chess engine", version::full_name());
    println!();
    println!("Usage:");
    println!("  aspet              show the starting position");
    println!("  aspet fen <FEN>    parse and display a FEN position");
    println!("  aspet --version    print version");
    println!("  aspet --help       print this help");
}
