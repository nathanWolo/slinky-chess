use cozy_chess::*;
// use core::time;
use std::str::FromStr;
use std::time::{Duration, Instant};
// use std::collections::HashMap;
const TT_SIZE: usize = 1 << 16;
pub struct AlphaBetaSearcher {
    transposition_table: [TTEntry; TT_SIZE],
    root_best_move: Move,
    root_score: i32,
    min_val: i32,
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

impl AlphaBetaSearcher {
    pub fn new() -> Self {
        AlphaBetaSearcher {
            // transposition_table: HashMap::new(),
            root_best_move: Move::from_str("a1a1").unwrap(),
            root_score: 0,
            min_val: -999,
            transposition_table: [TTEntry {
                hash: 0,
                depth: 0,
                score: 0,
                best_move: Move::from_str("a1a1").unwrap(),
                node_type: NodeType::Exact,
            }; TT_SIZE]
        }
    }

    fn count_material(&self, board: &Board, color: Color) -> i32 {
        let mut material: i32 = 0;
        let pieces: [Piece; 5] = [Piece::Pawn, Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen];
        let values: [i32; 5] = [1, 3, 3, 5, 9];
        for (i, piece) in pieces.iter().enumerate() {
            let count: i32 = board.colored_pieces(color, *piece).len() as i32;
            material += count * values[i];
        }
        material
    }

    fn evaluate(&self, board: &Board) -> i32 {
        let white_material = self.count_material(board, Color::White);
        let black_material = self.count_material(board, Color::Black);
        let mut score = white_material - black_material;
        if board.side_to_move() == Color::Black {
            score = -score;
        }
        score
    }

    fn negamax(&mut self, board: &Board, depth: i32, alpha: i32, beta: i32, ply:u32, start_time: Instant, time_limit: Duration) -> i32 {
        if board.status() != GameStatus::Ongoing {
            return match board.status() {
                GameStatus::Won => self.min_val + ply as i32,
                GameStatus::Drawn => 0,
                _ => 0,
            };
        }
        if depth == 0 {
            return self.evaluate(board);
        }
        if start_time.elapsed() > time_limit {
            return self.min_val;
        }

        // probe TT
        let mut best_score: i32 = -999;
        let mut new_alpha: i32 = alpha;
        let mut new_beta: i32 = beta;
        let entry: TTEntry = self.transposition_table[board.hash() as usize % TT_SIZE];
        if entry.hash == board.hash() && entry.depth >= depth && ply != 0{
            match entry.node_type {
                NodeType::Exact => return entry.score,
                NodeType::LowerBound => new_alpha = alpha.max(entry.score),
                NodeType::UpperBound => new_beta = beta.min(entry.score),
                // _ => (),
            }
            if new_alpha >= new_beta {
                return entry.score;
            }
        }
        board.generate_moves(|moves: PieceMoves| {
            for m in moves {
                let mut new_board: Board = board.clone();
                new_board.play(m);
                let score: i32 = -self.negamax(&new_board, depth - 1, -new_beta, -new_alpha, ply + 1, start_time, time_limit);
                if score > best_score {
                    best_score = score;
                    if (ply == 0) && (score.abs() != self.min_val.abs()) {
                        self.root_best_move = m;
                        self.root_score = score;
                    }
                }
                new_alpha = new_alpha.max(score);
                if new_alpha >= new_beta {
                    break;
                }
            }
            false
        });
        let node_type: NodeType = if best_score <= alpha {
            NodeType::UpperBound
        } else if best_score >= beta {
            NodeType::LowerBound
        } else {
            NodeType::Exact
        };
        let tt_entry: TTEntry = TTEntry {
            hash: board.hash(),
            depth: depth,
            score: best_score,
            best_move: self.root_best_move,
            node_type: node_type,
        };
        self.transposition_table[board.hash() as usize % TT_SIZE] = tt_entry;
        
        best_score
    }

    pub fn get_best_move(&mut self, board: &Board, thinking_time: u64) -> String {
        // let mut best_move = String::new();
        // let mut best_score = -1000;
        let start_time: Instant = Instant::now();
        let time_limit: Duration = Duration::from_millis(thinking_time);
        //do iterative deepening until we run out of time
        let mut current_depth: i32 = 1;
        let mut final_move = String::new();
        while start_time.elapsed() < time_limit {
            let score: i32 = self.negamax(board, current_depth, -1000, 1000, 0, start_time, time_limit);
            if score.abs() != self.min_val.abs() {
                // println!("info depth {} score {}", current_depth, score);
                final_move = self.root_best_move.clone().to_string();
            }
            current_depth += 1;
        }
        println!("info depth {} score {}", current_depth - 1, self.root_score);
        return final_move;
    }
}
