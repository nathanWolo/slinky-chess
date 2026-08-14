# Potential strength improvements

Each item is SPRTed **in isolation** against the current baseline. If H1 is accepted, the change is kept and becomes the new baseline. If H0 is accepted, it is reverted.

## Test protocol

Matches the existing `cutechess_commands.txt` setup as closely as this machine allows:

| Setting | Value |
|---|---|
| Book | `8mvs_big_+80_+109.epd` (same file as local testing) |
| TC | `4+0.04` |
| SPRT | `elo0=0 elo1=5 alpha=0.05 beta=0.05` |
| Games | 2 per opening, colors reversed (`-repeat`) |
| Concurrency | 4 (this VM; original command used 12) |
| Runner | `scripts/sprt.sh` via fastchess |

Baseline starts at `master` (`1101744`, 400MB hash), plus a protocol-only change that prints standard UCI castling (`e1g1` instead of cozy-chess `e1h1`) so fastchess accepts the move. That is not a playing-strength change.

## Pending

Ordered by expected Elo / confidence. One SPRT at a time.

### Search correctness (likely large)

- [ ] **En passant counted as a capture** — `piece_on(to)` is empty for EP, so EP is ordered/pruned as a quiet. Needed before any capture-based pruning touches EP.

### Move ordering / history

- [ ] **Two killer slots** — second killer with a smaller bonus.
- [ ] **Stronger history gravity** — malus of `depth²` (not `-1`) on quiet moves that failed to cut, with a clamp.

### Search extras

- [ ] **Late-move pruning** — skip remaining quiets at low depth after a move-count threshold.
- [ ] **Razoring** — at depth ≤ 2, if eval + margin < alpha, drop into qsearch and return on fail-low.
- [ ] **Internal iterative reduction** — reduce non-PV nodes with no TT move at depth ≥ 4.
- [ ] **NMP zugzwang guard** — skip null-move pruning in king+pawn endings; don't `unwrap()` `null_move()`.
- [ ] **Adaptive NMP reduction** — `R = 3 + (depth-4)/4` instead of a fixed `R = 3`.
- [ ] **Safer LMR** — do not reduce checks, in-check nodes, or (as aggressively) PV nodes.
- [ ] **TT probe/store in qsearch** — reuse deeper hits; depth-preferred replacement so qsearch cannot clobber them.
- [ ] **Qsearch promotions** — generate promotions, not only captures.
- [ ] **Qsearch delta pruning** — skip captures that cannot raise alpha even with a margin.
- [ ] **50-move draw detection** — return 0 at `halfmove_clock >= 100`.

### Time management

- [ ] **Use increment** — `winc`/`binc` are parsed and discarded. Soft/hard limits should include increment.
- [ ] **`go movetime` spends the allotted time** — current soft limit is `movetime/40`.

### Eval extras

- [ ] **Isolated pawn penalty**
- [ ] **Rook on the 7th** (enemy king on the 8th)

### Speed (NPS)

- [ ] **`play_unchecked` on generated legal moves**
- [ ] **ArrayVec for move scores** (moves already use ArrayVec)
- [ ] **Release LTO + `codegen-units = 1`**

## Accepted (in baseline)

- [x] **TT stores the node best move** — Elo difference: 82.9 +/- 21.7, LOS: 100.0 %, DrawRatio: 31.5 % SPRT: llr 2.95 (100.2%), lbound -2.94, ubound 2.94 — H1 was accepted (572 games, 4+0.04, `8mvs_big_+80_+109.epd`)
- [x] **Fix open-file detection** — Elo difference: 29.5 +/- 13.1, LOS: 100.0 %, DrawRatio: 36.1 % SPRT: llr 2.95 (100.2%), lbound -2.94, ubound 2.94 — H1 was accepted (1500 games, 4+0.04, `8mvs_big_+80_+109.epd`)
- [x] **Killers and history only on quiet cutoffs** — Elo difference: 35.5 +/- 14.4, LOS: 100.0 %, DrawRatio: 33.9 % SPRT: llr 2.95 (100.1%), lbound -2.94, ubound 2.94 — H1 was accepted (1228 games, 4+0.04, `8mvs_big_+80_+109.epd`)

## Rejected (H0)

- [x] **Qsearch searches check evasions** — did not pass H1. After 762 games: Elo -6.8 +/- 18.4, LOS 23.3 %, LLR -0.53. Not added.
- [x] **Age history instead of wiping** — did not pass H1. After 740 games: Elo +2.8 +/- 18.3, LOS 61.9 %, LLR 0.02. Not added.
