import subprocess, sys, chess

fen = sys.argv[1]
out = subprocess.run(["target/release/aspet", "makefen", fen],
                     capture_output=True, text=True, check=True).stdout
ours = dict(line.split(" ", 1) for line in out.splitlines())
ours = {k: v.strip() for k, v in ours.items()}

board = chess.Board(fen)
ref_moves = {m.uci() for m in board.legal_moves}
bad = 0
for m in board.legal_moves:
    board.push(m)
    ref = board.fen(en_passant="fen")   # always emit ep, like we do
    board.pop()
    got = ours.get(m.uci())
    if got != ref:
        bad += 1
        print(f"{m.uci()}\n  ours: {got}\n  ref:  {ref}")

missing = ref_moves - set(ours)
extra = set(ours) - ref_moves
if missing: print("missing moves:", sorted(missing))
if extra:   print("extra moves:", sorted(extra))
print("mismatches:", bad)