#!/usr/bin/env python3
"""Bisect a perft mismatch against Stockfish."""
import os, re, shutil, subprocess, sys
import chess

ASPET = os.environ.get("ASPET", "target/release/aspet")
SF = os.environ.get("STOCKFISH", "stockfish")

# e2e4: 20  /  a7a8q: 41  — ignores banners, info strings, and "total:".
MOVE_LINE = re.compile(r"^([a-h][1-8][a-h][1-8][nbrq]?)\s*:\s*(\d+)$")


def aspet_divide(fen, depth):
    out = subprocess.run([ASPET, "divide", str(depth), fen],
                         capture_output=True, text=True, check=True).stdout
    counts = {}
    for line in out.splitlines():
        m = MOVE_LINE.match(line.strip())
        if m:
            counts[m.group(1)] = int(m.group(2))
    if not counts:
        raise RuntimeError(f"aspet divide produced no moves:\n{out}")
    return counts


def sf_divide(fen, depth):
    p = subprocess.Popen([SF], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                         text=True, bufsize=1)

    def send(cmd):
        p.stdin.write(cmd + "\n")
        p.stdin.flush()

    def wait_for(token):
        for line in p.stdout:
            if line.strip() == token:
                return
        raise RuntimeError(f"stockfish exited before {token}")

    send("uci")
    wait_for("uciok")
    send("isready")
    wait_for("readyok")
    send(f"position fen {fen}")
    send(f"go perft {depth}")

    counts = {}
    for line in p.stdout:
        s = line.strip()
        if s.startswith("Nodes searched"):
            break
        m = MOVE_LINE.match(s)
        if m:
            counts[m.group(1)] = int(m.group(2))

    send("quit")
    p.wait()
    if not counts:
        raise RuntimeError("stockfish perft produced no moves")
    return counts


if not shutil.which(SF) and not os.path.isfile(SF):
    sys.exit(f"stockfish not found: {SF} (set STOCKFISH=/path/to/stockfish)")

fen, depth = sys.argv[1], int(sys.argv[2])
while depth >= 1:
    ours, ref = aspet_divide(fen, depth), sf_divide(fen, depth)
    bad = [k for k in sorted(set(ours) | set(ref)) if ours.get(k) != ref.get(k)]
    print(f"\n{fen}\ndepth {depth}: ours={sum(ours.values())} ref={sum(ref.values())}")
    for k in bad:
        print(f"  {k}: ours={ours.get(k)} ref={ref.get(k)}")
    if not bad:
        print("  clean at this depth")
        break
    if depth == 1:
        b = chess.Board(fen)
        ref_moves = {m.uci() for m in b.legal_moves}
        print("  missing:", sorted(ref_moves - set(ours)))
        print("  extra:  ", sorted(set(ours) - ref_moves))
        break
    b = chess.Board(fen)
    b.push(chess.Move.from_uci(bad[0]))
    fen, depth = b.fen(), depth - 1