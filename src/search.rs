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

    fn move_is_capture(&self, board: &Board, m: Move) -> bool {
        let occupant: Option<Piece> = board.piece_on(m.to);
        match occupant {
            Some(_) => true,
            None => false,
        }
    }

    fn score_moves(&self, _board: &Board, moves: Vec<Move>, tt_move: Move) -> Vec<i32> {
        //take in a board and a list of moves and return a list of scores for each move
        let mut scores: Vec<i32> = Vec::new();
        scores.reserve(moves.len());
        for m in moves {
            let mut score: i32 = 0;
            if m == tt_move {
                score += 100;
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
                score += target_value - attacker_value;
            }

            scores.push(score);
        }
        scores
    }

    fn sort_moves(&self, moves: Vec<Move>, scores: Vec<i32>) -> Vec<Move> {
        //take in a list of moves and a list of scores and return a list of moves sorted by score
        let mut zipped: Vec<(Move, i32)> = moves.into_iter().zip(scores.into_iter()).collect();
        zipped.sort_by(|a, b| b.1.cmp(&a.1));
        zipped.into_iter().map(|(m, _)| m).collect()
    }

    fn quiesce(&self, board: &Board, alpha: i32, beta: i32) -> i32 {
        let stand_pat: i32 = self.evaluate(board);
        if stand_pat >= beta {
            return beta;
        }

        let mut local_alpha: i32 = alpha.max(stand_pat);
        let mut captures: Vec<Move> = Vec::new();
        captures.reserve(16);
        board.generate_moves(|p: PieceMoves| {
            for m in p {
                if self.move_is_capture(board, m) {
                    captures.push(m);
                }
            }
            false
        });
        //sort moves
        let scores: Vec<i32> = self.score_moves(board, captures.clone(), Move::from_str("a1a1").unwrap());
        let sorted_moves: Vec<Move> = self.sort_moves(captures.clone(), scores);
        let mut best_score: i32 = -999;
        for m in sorted_moves {
            let mut new_board: Board = board.clone();
            new_board.play(m);
            let score: i32 = -self.quiesce(&new_board, -beta, -local_alpha);
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

    fn negamax(&mut self, board: &Board, depth: i32, alpha: i32, beta: i32, ply:u32, start_time: Instant, time_limit: Duration) -> i32 {
        if board.status() != GameStatus::Ongoing {
            match board.status() {
                GameStatus::Won => return self.min_val + ply as i32,
                GameStatus::Drawn => return 0,
                _ => (),
            };
        }
        if depth == 0 {
            // return self.evaluate(board);
            return self.quiesce(board, alpha, beta);
        }
        if start_time.elapsed() > time_limit {
            return self.min_val;
        }

        // probe TT
        let mut best_score: i32 = -999;
        let mut new_alpha: i32 = alpha;
        let mut new_beta: i32 = beta;
        let entry: TTEntry = self.transposition_table[board.hash() as usize % TT_SIZE];
        let tt_move: Move = entry.best_move;
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
        //generate all moves and store them in a vector
        let mut moves: Vec<Move> = Vec::new();
        moves.reserve(32);
        board.generate_moves(|p: PieceMoves| {
            for m in p {
                moves.push(m);
            }
            false
        });
        let scores: Vec<i32> = self.score_moves(board, moves.clone(), tt_move);
        let sorted_moves: Vec<Move> = self.sort_moves(moves.clone(), scores);

        //search through all moves
        for m in sorted_moves {
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
