#!/usr/bin/env bash
# Isolated SPRT: dev vs baseline, same bounds/book/TC as cutechess_commands.txt
set -euo pipefail

if [[ $# -lt 3 ]]; then
    echo "usage: $0 <dev-binary> <baseline-binary> <run-name>" >&2
    exit 1
fi

DEV="$1"
BASE="$2"
NAME="$3"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BOOK="${BOOK:-$ROOT/books/8mvs_big_+80_+109.epd}"
FASTCHESS="${FASTCHESS:-$ROOT/tools/fastchess-linux-x86-64/fastchess}"
OUT_DIR="$ROOT/sprt"
mkdir -p "$OUT_DIR"

exec "$FASTCHESS" \
    -engine name=dev cmd="$DEV" \
    -engine name=baseline cmd="$BASE" \
    -each proto=uci tc=4+0.04 \
    -openings file="$BOOK" format=epd order=random \
    -repeat \
    -sprt elo0=0 elo1=5 alpha=0.05 beta=0.05 \
    -concurrency 4 \
    -rounds 100000 \
    -ratinginterval 10 \
    -draw movenumber=80 movecount=8 score=15 \
    -pgnout file="$OUT_DIR/${NAME}.pgn" \
    -log file="$OUT_DIR/${NAME}.log"
