import sys
import chess

def perft(board, depth):
    if depth == 0:
        return 1
    moves = list(board.legal_moves)
    if depth == 1:
        return len(moves)
    total = 0
    for m in moves:
        board.push(m)
        total += perft(board, depth - 1)
        board.pop()
    return total

fen, depth = sys.argv[1], int(sys.argv[2])
board = chess.Board(fen)
total = 0
for m in sorted(board.legal_moves, key=lambda m: m.uci()):
    board.push(m)
    count = perft(board, depth - 1)
    board.pop()
    print(f"{m.uci()}: {count}")
    total += count
print(f"total: {total}")