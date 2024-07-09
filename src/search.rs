use cozy_chess::*;
// use core::time;
use std::str::FromStr;
use std::time::{Duration, Instant};
// use std::collections::HashMap;
const TT_SIZE: usize = 1 << 22;
pub struct AlphaBetaSearcher {
    transposition_table: Vec<TTEntry>,
    root_best_move: Move,
    root_score: i32,
    min_val: i32,
    nodes: u64,
    killer_table: Vec<Move>,
    history_table: Vec<Vec<Vec<i32>>>,
}
#[derive(Clone, Copy)]
struct TTEntry {
    hash: u64,
    depth: i32,
    score: i32,
    best_move: Move,
    node_type: NodeType,
}
#[derive(Clone, Copy)]
enum NodeType {
    Exact,
    LowerBound,
    UpperBound,
}
const TT_BONUS: i32 = 1 << 24;
const CAPTURE_BONUS: i32 = 1 << 20;
const KILLER_BONUS: i32 = 1 << 20;

const MG_PAWN_TABLE: [i32; 64] =      
[ 0,   0,   0,   0,   0,   0,  0,   0,
98, 134,  61,  95,  68, 126, 34, -11,
-6,   7,  26,  31,  65,  56, 25, -20,
-14,  13,   6,  21,  23,  12, 17, -23,
-27,  -2,  -5,  12,  17,   6, 10, -25,
-26,  -4,  -4, -10,   3,   3, 33, -12,
-35,  -1, -20, -23, -15,  24, 38, -22,
 0,   0,   0,   0,   0,   0,  0,   0,];
const EG_PAWN_TABLE: [i32; 64] =      
[ 0,   0,   0,   0,   0,   0,   0,   0,
178, 173, 158, 134, 147, 132, 165, 187,
 94, 100,  85,  67,  56,  53,  82,  84,
 32,  24,  13,   5,  -2,   4,  17,  17,
 13,   9,  -3,  -7,  -7,  -8,   3,  -1,
  4,   7,  -6,   1,   0,  -5,  -1,  -8,
 13,   8,   8,  10,  13,   0,   2,  -7,
  0,   0,   0,   0,   0,   0,   0,   0,
];

const MG_KNIGHT_TABLE: [i32; 64] =    
[ -167, -89, -34, -49,  61, -97, -15, -107,
-73, -41,  72,  36,  23,  62,   7,  -17,
-47,  60,  37,  65,  84, 129,  73,   44,
 -9,  17,  19,  53,  37,  69,  18,   22,
-13,   4,  16,  13,  28,  19,  21,   -8,
-23,  -9,  12,  10,  19,  17,  25,  -16,
-29, -53, -12,  -3,  -1,  18, -14,  -19,
-105, -21, -58, -33, -17, -28, -19,  -23,];

const EG_KNIGHT_TABLE: [i32; 64] =    
[ -58, -38, -13, -28, -31, -27, -63, -99,
-25,  -8, -25,  -2,  -9, -25, -24, -52,
-24, -20,  10,   9,  -1,  -9, -19, -41,
-17,   3,  22,  22,  22,  11,   8, -18,
-18,  -6,  16,  25,  16,  17,   4, -18,
-23,  -3,  -1,  15,  10,  -3, -20, -22,
-42, -20, -10,  -5,  -2, -20, -23, -44,
-29, -51, -23, -15, -22, -18, -50, -64,];

const MG_BISHOP_TABLE: [i32; 64] =    
[ -29,   4, -82, -37, -25, -42,   7,  -8,
-26,  16, -18, -13,  30,  59,  18, -47,
-16,  37,  43,  40,  35,  50,  37,  -2,
 -4,   5,  19,  50,  37,  37,   7,  -2,
 -6,  13,  13,  26,  34,  12,  10,   4,
  0,  15,  15,  15,  14,  27,  18,  10,
  4,  15,  16,   0,   7,  21,  33,   1,
-33,  -3, -14, -21, -13, -12, -39, -21,];

const EG_BISHOP_TABLE: [i32; 64] =    [    -14, -21, -11,  -8, -7,  -9, -17, -24,
-8,  -4,   7, -12, -3, -13,  -4, -14,
 2,  -8,   0,  -1, -2,   6,   0,   4,
-3,   9,  12,   9, 14,  10,   3,   2,
-6,   3,  13,  19,  7,  10,  -3,  -9,
-12,  -3,   8,  10, 13,   3,  -7, -15,
-14, -18,  -7,  -1,  4,  -9, -15, -27,
-23,  -9, -23,  -5, -9, -16,  -5, -17,];

const MG_ROOK_TABLE: [i32; 64] =      [     32,  42,  32,  51, 63,  9,  31,  43,
27,  32,  58,  62, 80, 67,  26,  44,
-5,  19,  26,  36, 17, 45,  61,  16,
-24, -11,   7,  26, 24, 35,  -8, -20,
-36, -26, -12,  -1,  9, -7,   6, -23,
-45, -25, -16, -17,  3,  0,  -5, -33,
-44, -16, -20,  -9, -1, 11,  -6, -71,
-19, -13,   1,  17, 16,  7, -37, -26,];

const EG_ROOK_TABLE: [i32; 64] = [    13, 10, 18, 15, 12,  12,   8,   5,
11, 13, 13, 11, -3,   3,   8,   3,
 7,  7,  7,  5,  4,  -3,  -5,  -3,
 4,  3, 13,  1,  2,   1,  -1,   2,
 3,  5,  8,  4, -5,  -6,  -8, -11,
-4,  0, -5, -1, -7, -12,  -8, -16,
-6, -6,  0,  2, -9,  -9, -11,  -3,
-9,  2,  3, -1, -5, -13,   4, -20,];

const MG_QUEEN_TABLE: [i32; 64] =     [    -28,   0,  29,  12,  59,  44,  43,  45,
-24, -39,  -5,   1, -16,  57,  28,  54,
-13, -17,   7,   8,  29,  56,  47,  57,
-27, -27, -16, -16,  -1,  17,  -2,   1,
 -9, -26,  -9, -10,  -2,  -4,   3,  -3,
-14,   2, -11,  -2,  -5,   2,  14,   5,
-35,  -8,  11,   2,   8,  15,  -3,   1,
 -1, -18,  -9,  10, -15, -25, -31, -50,];

 const EG_QUEEN_TABLE: [i32; 64] =     [     -9,  22,  22,  27,  27,  19,  10,  20,
 -17,  20,  32,  41,  58,  25,  30,   0,
 -20,   6,   9,  49,  47,  35,  19,   9,
   3,  22,  24,  45,  57,  40,  57,  36,
 -18,  28,  19,  47,  31,  34,  39,  23,
 -16, -27,  15,   6,   9,  17,  10,   5,
 -22, -23, -30, -16, -16, -23, -36, -32,
 -33, -28, -22, -43,  -5, -32, -20, -41,];

 const MG_KING_TABLE: [i32; 64] =      [    -65,  23,  16, -15, -56, -34,   2,  13,
 29,  -1, -20,  -7,  -8,  -4, -38, -29,
 -9,  24,   2, -16, -20,   6,  22, -22,
-17, -20, -12, -27, -30, -25, -14, -36,
-49,  -1, -27, -39, -46, -44, -33, -51,
-14, -14, -22, -46, -44, -30, -15, -27,
  1,   7,  -8, -64, -43, -16,   9,   8,
-15,  36,  12, -54,   8, -28,  24,  14,];

const EG_KING_TABLE: [i32; 64] =     [    -74, -35, -18, -18, -11,  15,   4, -17,
-12,  17,  14,  17,  17,  38,  23,  11,
 10,  17,  23,  15,  20,  45,  44,  13,
 -8,  22,  24,  27,  26,  33,  26,   3,
-18,  -4,  21,  24,  27,  23,   9, -11,
-19,  -3,  11,  21,  23,  16,   7,  -9,
-27, -11,   4,  13,  14,   4,  -5, -17,
-53, -34, -21, -11, -28, -14, -24, -43];

const MG_PAWN_MATERIAL: i32 = 82;
const MG_KNIGHT_MATERIAL: i32 = 337;
const MG_BISHOP_MATERIAL: i32 = 365;
const MG_ROOK_MATERIAL: i32 = 477;
const MG_QUEEN_MATERIAL: i32 = 1025;
const MG_KING_MATERIAL: i32 = 0;
const EG_PAWN_MATERIAL: i32 = 94;
const EG_KNIGHT_MATERIAL: i32 = 281;
const EG_BISHOP_MATERIAL: i32 = 297;
const EG_ROOK_MATERIAL: i32 = 512;
const EG_QUEEN_MATERIAL: i32 = 936;
const EG_KING_MATERIAL: i32 = 0;

impl AlphaBetaSearcher {
    pub fn new() -> Self {
        AlphaBetaSearcher {
            root_best_move: Move::from_str("a1a1").unwrap(),
            root_score: 0,
            min_val: - (1 << 30),
            transposition_table: vec![TTEntry {
                hash: 0,
                depth: 0,
                score: 0,
                best_move: Move::from_str("a1a1").unwrap(),
                node_type: NodeType::Exact,
            }; TT_SIZE],
            killer_table: vec![Move::from_str("a1a1").unwrap(); 128],
            history_table: vec![vec![vec![0; 64]; 64]; 2],
            nodes: 0,
        }
    }

    // fn count_material(&self, board: &Board, color: Color) -> i32 {
    //     let mut material: i32 = 0;
    //     let pieces: [Piece; 5] = [Piece::Pawn, Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen];
    //     let values: [i32; 5] = [100, 300, 300, 500, 900];
    //     for (i, piece) in pieces.iter().enumerate() {
    //         let count: i32 = board.colored_pieces(color, *piece).len() as i32;
    //         material += count * values[i];
    //     }
    //     material
    // }
    // fn check_two_bishops(&self, board: &Board, color: Color) -> bool {
    //     let bishops = board.colored_pieces(color, Piece::Bishop);
    //     return bishops.len() >= 2;
    // }
    // fn check_rooks_same_file(&self, board: &Board, color: Color) -> bool {
    //     let rooks = board.colored_pieces(color, Piece::Rook);
    //     if rooks.len() < 2 {
    //         return false;
    //     }
    //     let files: Vec<File> = rooks.iter().map(|s| s.file()).collect();
    //     for i in 0..files.len() {
    //         for j in i + 1..files.len() {
    //             if files[i] == files[j] {
    //                 return true;
    //             }
    //         }
    //     }
    //     false
    // }
    // fn pawn_advancement_score(&self, board: &Board, color: Color) -> i32 {
    //     let mut score: i32 = 0;
    //     let pawns = board.colored_pieces(color, Piece::Pawn);
    //     for p in pawns {
    //         let rank: Rank = p.rank();
    //         let rank_val: i32 = match rank {
    //             Rank::First => 0,
    //             Rank::Second => 1,
    //             Rank::Third => 2,
    //             Rank::Fourth => 3,
    //             Rank::Fifth => 4,
    //             Rank::Sixth => 5,
    //             Rank::Seventh => 6,
    //             Rank::Eighth => 7,
    //         };
    //         if color == Color::White {
    //             score += rank_val;
    //         } else {
    //             score += 7 - rank_val;
    //         }
    //     }
    //     score * 2
    // }
    // fn simple_evaluate(&self, board: &Board) -> i32 {
    //     let mut white_score: i32 = 0;
    //     let mut black_score: i32 = 0;
    //     white_score += self.count_material(board, Color::White);
    //     black_score += self.count_material(board, Color::Black);
    //     if self.check_two_bishops(board, Color::White) {
    //         white_score += 30;
    //     }
    //     if self.check_two_bishops(board, Color::Black) {
    //         black_score += 30;
    //     }
    //     if self.check_rooks_same_file(board, Color::White) {
    //         white_score += 20;
    //     }
    //     if self.check_rooks_same_file(board, Color::Black) {
    //         black_score += 20;
    //     }
    //     white_score += self.pawn_advancement_score(board, Color::White);
    //     black_score += self.pawn_advancement_score(board, Color::Black);
    //     let mut score: i32 = white_score - black_score;
    //     if board.side_to_move() == Color::Black {
    //         score = -score;
    //     }
    //     score
    // }

    pub fn pesto_evaluate(&self, board: &Board) -> i32 {
        let mut white_mg:i32 = 0;
        let mut black_mg:i32 = 0;
        let mut white_eg:i32 = 0;
        let mut black_eg:i32 = 0;
        let piece_types: [Piece; 6] = [Piece::Pawn, Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen, Piece::King];
        let mut mg_phase: i32 = 0;
        let mut score: i32;
        for piece in piece_types.iter() {
            let white: BitBoardIter = board.colored_pieces(Color::White, *piece).iter();
            let black: BitBoardIter = board.colored_pieces(Color::Black, *piece).iter();
            for square in white {
                let rel_square: usize = square.relative_to(Color::Black) as usize; //this flips the perspective because the PSTs are stored in A8, B8, C8 ... order
                match piece {
                    Piece::Pawn => white_mg += MG_PAWN_TABLE[rel_square] + MG_PAWN_MATERIAL,
                    Piece::Knight => white_mg += MG_KNIGHT_TABLE[rel_square] + MG_KNIGHT_MATERIAL,
                    Piece::Bishop => white_mg += MG_BISHOP_TABLE[rel_square] + MG_BISHOP_MATERIAL,
                    Piece::Rook => white_mg += MG_ROOK_TABLE[rel_square] + MG_ROOK_MATERIAL,
                    Piece::Queen => white_mg += MG_QUEEN_TABLE[rel_square] + MG_QUEEN_MATERIAL,
                    Piece::King => white_mg += MG_KING_TABLE[rel_square] + MG_KING_MATERIAL,
                }
                match piece {
                    Piece::Pawn => white_eg += EG_PAWN_TABLE[rel_square] + EG_PAWN_MATERIAL,
                    Piece::Knight => white_eg += EG_KNIGHT_TABLE[rel_square] + EG_KNIGHT_MATERIAL,
                    Piece::Bishop => white_eg += EG_BISHOP_TABLE[rel_square] + EG_BISHOP_MATERIAL,
                    Piece::Rook => white_eg += EG_ROOK_TABLE[rel_square] + EG_ROOK_MATERIAL,
                    Piece::Queen => white_eg += EG_QUEEN_TABLE[rel_square] + EG_QUEEN_MATERIAL,
                    Piece::King => white_eg += EG_KING_TABLE[rel_square] + EG_KING_MATERIAL,
                }                    
                //control mg vs eg phase
                match piece {
                    Piece::Pawn => mg_phase += 0,
                    Piece::Knight => mg_phase += 1,
                    Piece::Bishop => mg_phase += 1,
                    Piece::Rook => mg_phase += 2,
                    Piece::Queen => mg_phase += 4,
                    _ => (),
                }
            }
            for square in black {
                let rel_square: usize = square as usize;
                match piece {
                    Piece::Pawn => black_mg += MG_PAWN_TABLE[rel_square] + MG_PAWN_MATERIAL,
                    Piece::Knight => black_mg += MG_KNIGHT_TABLE[rel_square] + MG_KNIGHT_MATERIAL,
                    Piece::Bishop => black_mg += MG_BISHOP_TABLE[rel_square] + MG_BISHOP_MATERIAL,
                    Piece::Rook => black_mg += MG_ROOK_TABLE[rel_square] + MG_ROOK_MATERIAL,
                    Piece::Queen => black_mg += MG_QUEEN_TABLE[rel_square] + MG_QUEEN_MATERIAL,
                    Piece::King => black_mg += MG_KING_TABLE[rel_square] + MG_KING_MATERIAL,
                
                }
                match piece {
                    Piece::Pawn => black_eg += EG_PAWN_TABLE[rel_square] + EG_PAWN_MATERIAL,
                    Piece::Knight => black_eg += EG_KNIGHT_TABLE[rel_square] + EG_KNIGHT_MATERIAL,
                    Piece::Bishop => black_eg += EG_BISHOP_TABLE[rel_square] + EG_BISHOP_MATERIAL,
                    Piece::Rook => black_eg += EG_ROOK_TABLE[rel_square] + EG_ROOK_MATERIAL,
                    Piece::Queen => black_eg += EG_QUEEN_TABLE[rel_square] + EG_QUEEN_MATERIAL,
                    Piece::King => black_eg += EG_KING_TABLE[rel_square] + EG_KING_MATERIAL,

                }
                //material value
                //control mg vs eg phase
                match piece {
                    Piece::Pawn => mg_phase += 0,
                    Piece::Knight => mg_phase += 1,
                    Piece::Bishop => mg_phase += 1,
                    Piece::Rook => mg_phase += 2,
                    Piece::Queen => mg_phase += 4,
                    _ => (),
                }
            }
        }
        let mg: i32 = white_mg - black_mg;
        let eg: i32 = white_eg - black_eg;
        mg_phase = mg_phase.min(24);
        let eg_phase: i32 = 24 - mg_phase;
        score = (mg * mg_phase + eg * eg_phase)/ 24;
        // score = mg;
        if board.side_to_move() == Color::Black {
            score = -score;
        }
        score
    }

    fn move_is_capture(&self, board: &Board, m: &Move) -> bool {
        let occupant: Option<Piece> = board.piece_on(m.to);
        match occupant {
            Some(_) => true,
            None => false,
        }
    }

    fn score_moves(&self, _board: &Board, moves: &Vec<Move>, tt_move: Move, ply: u32) -> Vec<i32> {
        //take in a board and a list of moves and return a list of scores for each move
        let mut scores: Vec<i32> = Vec::new();
        scores.reserve(moves.len());
        // let tt_bonus: i32 = 1 << 24;
        // let capture_bonus: i32 = 1 << 20;
        // let killer_bonus: i32 = 1 << 20;
        for m in moves {
            let mut score: i32 = 0;
            if *m == tt_move {
                score += TT_BONUS;
            }
            // Most valuable victim - least valuable attacker
            if self.move_is_capture(_board, m) {
                let target: Option<Piece> = _board.piece_on(m.to);
                let attacker: Piece = _board.piece_on(m.from).unwrap();
                let attacker_value: i32 = match attacker {
                    Piece::Pawn => 1,
                    Piece::Knight => 3,
                    Piece::Bishop => 3,
                    Piece::Rook => 5,
                    Piece::Queen => 9,
                    _ => 0,
                };
                let target_value: i32 = match target.unwrap() {
                    Piece::Pawn => 1,
                    Piece::Knight => 3,
                    Piece::Bishop => 3,
                    Piece::Rook => 5,
                    Piece::Queen => 9,
                    _ => 0,
                };
                score += target_value *20 - attacker_value + CAPTURE_BONUS;
            }
            else if *m == self.killer_table[ply as usize] {
                score += KILLER_BONUS; //TODO: revisit this constant
            }
            else {
                score += self.history_table[_board.side_to_move() as usize][m.from as usize][m.to as usize];
            }
            scores.push(score);
        }
        scores
    }

    fn sort_moves(&self, moves: &mut Vec<Move>, scores: &mut Vec<i32>) {
        let mut i = 1;
        while i < moves.len() {
            let mut j = i;
            while j > 0 && scores[j] > scores[j - 1] {
                scores.swap(j, j - 1);
                moves.swap(j, j - 1);
                j -= 1;
            }
            i += 1;
        }
    }

    fn quiesce(&mut self, board: &Board, alpha: i32, beta: i32, ply: u32, start_time: Instant, time_limit: Duration) -> i32 {
        //quiesce the position
        self.nodes += 1;
        let stand_pat: i32 = self.pesto_evaluate(board);
        if stand_pat >= beta {
            return beta;
        }
        if start_time.elapsed() > time_limit {
            return self.min_val;
        }

        let mut local_alpha: i32 = alpha.max(stand_pat);
        let mut moves: Vec<Move> = Vec::new();
        moves.reserve(16);
        board.generate_moves(|p: PieceMoves| {
            for m in p {
                if self.move_is_capture(board, &m) {
                    moves.push(m);
                }
            }
            false
        });
        //sort moves
        let mut scores: Vec<i32> = self.score_moves(board, &moves, Move::from_str("a1a1").unwrap(), ply);
        self.sort_moves(&mut moves, &mut scores);
        let mut best_score: i32 = -self.min_val;
        for m in moves {
            let mut new_board: Board = board.clone();
            new_board.play(m);
            let score: i32 = -self.quiesce(&new_board, -beta, -local_alpha, ply + 1, start_time, time_limit);
            if score >= beta {
                return beta;
            }
            if score > best_score {
                best_score = score;
                local_alpha = local_alpha.max(score);
            }
        }
        local_alpha
    }

    fn pvs(&mut self, board: &Board, depth: i32, alpha: i32, beta: i32, ply:u32, start_time: Instant, time_limit: Duration, mut extend_checks: bool) -> i32 {
        self.nodes += 1;
        if board.status() != GameStatus::Ongoing {
            match board.status() {
                GameStatus::Won => return self.min_val + (ply as i32),
                GameStatus::Drawn => return 0,
                _ => (),
            };
        }

        //check extension: if in check, increase depth by 1
        let mut depth_modifier: i32 = 0;
        if board.checkers().len() > 0  && ply != 0 && extend_checks{
            depth_modifier += 1;
            extend_checks = false;
        }

        if depth == 0  && depth_modifier == 0{
            return self.quiesce(board, alpha, beta, ply, start_time, time_limit);
        }
        if start_time.elapsed() > time_limit {
            return self.min_val;
        }
        let pv_node: bool = beta - alpha > 1;
        // probe TT
        let mut best_score: i32 = self.min_val;
        let mut new_alpha: i32 = alpha;
        let mut new_beta: i32 = beta;
        let entry: TTEntry = self.transposition_table[board.hash() as usize % TT_SIZE];
        let tt_move: Move = entry.best_move;
        if entry.hash == board.hash() && entry.depth >= depth && ply != 0 && !pv_node {
            match entry.node_type {
                NodeType::Exact => return entry.score,
                NodeType::LowerBound => new_alpha = alpha.max(entry.score),
                NodeType::UpperBound => new_beta = beta.min(entry.score),
            }
            if new_alpha >= new_beta {
                return entry.score;
            }
        }
        //generate all moves and store them in a vector
        let mut moves: Vec<Move> = Vec::new();
        moves.reserve(32);
        board.generate_moves(|p: PieceMoves| {
            for m in p {
                moves.push(m);
            }
            false
        });
        let mut scores: Vec<i32> = self.score_moves(board, &moves, tt_move, ply);
        self.sort_moves(&mut moves, &mut scores);
        let mut score: i32;

        let mut new_board = board.clone();
        for (i, m) in moves.iter().enumerate() {
            // let mut new_board: Board = board.clone();
            new_board.play(*m);
            if i == 0 { //principal variation
                score = -self.pvs(&new_board, depth - 1 + depth_modifier, -new_beta, -new_alpha, ply + 1, start_time, time_limit, extend_checks);
            }
            else {
                score = -self.pvs(&new_board, depth - 1 + depth_modifier, -new_alpha - 1, -new_alpha, ply + 1, start_time, time_limit, extend_checks);
                if new_alpha < score && score < new_beta {
                    score = -self.pvs(&new_board, depth - 1 + depth_modifier, -new_beta, -score, ply + 1, start_time, time_limit, extend_checks);
                }
            }
            new_board = board.clone();
            if score > best_score {
                best_score = score;
                if (ply == 0) && (score.abs() != self.min_val.abs()) {
                    self.root_best_move = *m;
                    self.root_score = score;
                }
            }
            new_alpha = new_alpha.max(score);
            if new_alpha >= new_beta {
                self.killer_table[ply as usize] = *m;
                self.history_table[board.side_to_move() as usize][m.from as usize][m.to as usize] += depth * depth;
                break;
            }
        }
        let node_type: NodeType = if best_score <= alpha {
            NodeType::UpperBound
        } else if best_score >= beta {
            NodeType::LowerBound
        } else {
            NodeType::Exact
        };
        //idea for later: dont store in TT if score is timeout
        if best_score.abs() != self.min_val.abs() {
            let tt_entry: TTEntry = TTEntry {
                hash: board.hash(),
                depth,
                score: best_score,
                best_move: self.root_best_move,
                node_type,
            };
            self.transposition_table[board.hash() as usize % TT_SIZE] = tt_entry;
        }
        
        best_score
    }

    pub fn get_best_move(&mut self, board: &Board, thinking_time: u64) -> String {
        let start_time: Instant = Instant::now();
        let time_limit: Duration = Duration::from_millis(thinking_time);
        //do iterative deepening until we run out of time
        let mut current_depth: i32 = 1;
        let final_move: String;
        self.nodes = 0;
        self.root_best_move = Move::from_str("a1a1").unwrap();
        //clear history table
        self.history_table = vec![vec![vec![0; 64]; 64]; 2];

        let mut aspiration_window: i32 = 15;
        let mut alpha: i32 = -99999999;
        let mut beta: i32 = 99999999;

        while start_time.elapsed() < time_limit && current_depth < 100 {
            let score: i32 = self.pvs(board, current_depth, alpha, beta, 0, start_time, time_limit, true);
            if score <= alpha || score >= beta {
                //fail high or low, re-search with gradual widening
                aspiration_window *= 2;
                alpha = score - aspiration_window;
                beta = score + aspiration_window;
                continue;
            }
            aspiration_window = 15;
            alpha = score - aspiration_window;
            beta = score + aspiration_window;
            println!("depth {} score cp {} NPS {}k", current_depth, score, (self.nodes as f32) / (start_time.elapsed().as_secs_f32() *1000.0));
            current_depth += 1;
        }
        final_move = self.root_best_move.clone().to_string();
        //check if final_move is legal
        let mut legal_moves: Vec<String> = Vec::new();
        board.generate_moves(|p: PieceMoves| {
            for m in p {
                legal_moves.push(m.to_string());
            }
            false
        });
        //if move is not legal, print move and fen to stderr and panic
        if !legal_moves.contains(&final_move) {
            panic!("Illegal move {} in position {}. Searched to depth {} with root_best_move {}", final_move, board, current_depth - 1, self.root_best_move);
        }
        println!("info depth {} score cp {} NPS {}k", current_depth - 1, self.root_score, (self.nodes as f32) / (start_time.elapsed().as_secs_f32() *1000.0));
        return final_move;
    }
}
