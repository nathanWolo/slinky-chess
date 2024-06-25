use cozy_chess::*;
// use core::time;
use std::str::FromStr;
use std::time::{Duration, Instant};
// use std::collections::HashMap;

pub struct AlphaBetaSearcher {
    // transposition_table: HashMap<u64, i32>, // Example of long-term state
    root_best_move: Move,
    root_score: i32,
    min_val: i32,
}

impl AlphaBetaSearcher {
    pub fn new() -> Self {
        AlphaBetaSearcher {
            // transposition_table: HashMap::new(),
            root_best_move: Move::from_str("a1a1").unwrap(),
            root_score: 0,
            min_val: -999,
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

        // Transposition table lookup (example, assuming Board implements hash function)
        // let board_hash = board.hash();
        // if let Some(&score) = self.transposition_table.get(&board_hash) {
        //     return score;
        // }

        let mut best_score = -999;
        let mut new_alpha = alpha;
        board.generate_moves(|moves: PieceMoves| {
            for m in moves {
                let mut new_board: Board = board.clone();
                new_board.play(m);
                let score: i32 = -self.negamax(&new_board, depth - 1, -beta, -new_alpha, ply + 1, start_time, time_limit);
                if score > best_score {
                    best_score = score;
                    if (ply == 0) && (score.abs() != self.min_val.abs()) {
                        self.root_best_move = m;
                        self.root_score = score;
                    }
                }
                new_alpha = new_alpha.max(score);
                if new_alpha >= beta {
                    break;
                }
            }
            false
        });

        // Store result in transposition table
        // self.transposition_table.insert(board_hash, best_score);
        
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
