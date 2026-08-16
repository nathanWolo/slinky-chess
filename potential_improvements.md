# Potential strength improvements

Each item is SPRTed **in isolation** against the current baseline. If H1 is accepted, the change is kept and becomes the new baseline. If H0 is accepted, it is reverted.

## Test protocol

Matches the existing `cutechess_commands.txt` setup as closely as this machine allows:

| Setting | Value |
|---|---|
| Book | `8mvs_big_+80_+109.epd` (same file as local testing) |
| TC | `4+0.04` (override with `TC=`, e.g. `TC=6+0.06`) |
| SPRT | `elo0=0 elo1=5 alpha=0.05 beta=0.05` |
| Games | 2 per opening, colors reversed (`-repeat`) |
| Concurrency | 4 (this VM; original command used 12) |
| Runner | `scripts/sprt.sh` via fastchess |

Baseline starts at `master` (`1101744`, 400MB hash), plus a protocol-only change that prints standard UCI castling (`e1g1` instead of cozy-chess `e1h1`) so fastchess accepts the move. That is not a playing-strength change.

## Pending

Ordered by expected Elo / confidence. One SPRT at a time.

### Move ordering / history

- [ ] **Two killer slots** — second killer with a smaller bonus.
- [ ] **Stronger history gravity** — malus of `depth²` (not `-1`) on quiet moves that failed to cut, with a clamp.

### Search extras

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

To continue: build two binaries, then `scripts/sprt.sh ./bin/dev ./bin/baseline <name>`. Keep one pending item in the working tree at a time. If H1 is accepted, commit it and copy the dev binary over `bin/baseline`. Otherwise revert the patched files.

## Accepted (in baseline)

- [x] **TT stores the node best move** — Elo difference: 82.9 +/- 21.7, LOS: 100.0 %, DrawRatio: 31.5 % SPRT: llr 2.95 (100.2%), lbound -2.94, ubound 2.94 — H1 was accepted (572 games, 4+0.04, `8mvs_big_+80_+109.epd`)
- [x] **Fix open-file detection** — Elo difference: 29.5 +/- 13.1, LOS: 100.0 %, DrawRatio: 36.1 % SPRT: llr 2.95 (100.2%), lbound -2.94, ubound 2.94 — H1 was accepted (1500 games, 4+0.04, `8mvs_big_+80_+109.epd`)
- [x] **Killers and history only on quiet cutoffs** — Elo difference: 35.5 +/- 14.4, LOS: 100.0 %, DrawRatio: 33.9 % SPRT: llr 2.95 (100.1%), lbound -2.94, ubound 2.94 — H1 was accepted (1228 games, 4+0.04, `8mvs_big_+80_+109.epd`)
- [x] **En passant counted as a capture** — kept as a correctness bugfix (EP was ordered and pruned as a quiet). SPRT did not reach H0/H1. 4+0.04: Elo -0.4 +/- 15.4, 920 games, LLR -0.23. 6+0.06: Elo 1.12 +/- 4.77, LOS 67.8 %, 9918 games, LLR -1.17. Promoted to baseline anyway.

## Rejected (H0)

- [x] **Qsearch searches check evasions** — did not pass H1. After 762 games: Elo -6.8 +/- 18.4, LOS 23.3 %, LLR -0.53. Not added.
- [x] **Age history instead of wiping** — did not pass H1. After 740 games: Elo +2.8 +/- 18.3, LOS 61.9 %, LLR 0.02. Not added.
- [x] **Late-move pruning** (`depth <= 4`, skip quiets after `3 + depth²`) — H0 accepted. Elo -66.4 +/- 20.5, LOS 0.0 %, 710 games. Not added.
