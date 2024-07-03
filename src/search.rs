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
            nodes: 0,
        }
    }

    fn count_material(&self, board: &Board, color: Color) -> i32 {
        let mut material: i32 = 0;
        let pieces: [Piece; 5] = [Piece::Pawn, Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen];
        let values: [i32; 5] = [100, 300, 300, 500, 900];
        for (i, piece) in pieces.iter().enumerate() {
            let count: i32 = board.colored_pieces(color, *piece).len() as i32;
            material += count * values[i];
        }
        material
    }
    fn check_two_bishops(&self, board: &Board, color: Color) -> bool {
        let bishops = board.colored_pieces(color, Piece::Bishop);
        return bishops.len() >= 2;
    }
    fn check_rooks_same_file(&self, board: &Board, color: Color) -> bool {
        let rooks = board.colored_pieces(color, Piece::Rook);
        if rooks.len() < 2 {
            return false;
        }
        let files: Vec<File> = rooks.iter().map(|s| s.file()).collect();
        for i in 0..files.len() {
            for j in i + 1..files.len() {
                if files[i] == files[j] {
                    return true;
                }
            }
        }
        false
    }
    fn pawn_advancement_score(&self, board: &Board, color: Color) -> i32 {
        let mut score: i32 = 0;
        let pawns = board.colored_pieces(color, Piece::Pawn);
        for p in pawns {
            let rank: Rank = p.rank();
            let rank_val: i32 = match rank {
                Rank::First => 0,
                Rank::Second => 1,
                Rank::Third => 2,
                Rank::Fourth => 3,
                Rank::Fifth => 4,
                Rank::Sixth => 5,
                Rank::Seventh => 6,
                Rank::Eighth => 7,
            };
            if color == Color::White {
                score += rank_val;
            } else {
                score += 7 - rank_val;
            }
        }
        score * 2
    }
    fn evaluate(&self, board: &Board) -> i32 {
        let mut white_score: i32 = 0;
        let mut black_score: i32 = 0;
        white_score += self.count_material(board, Color::White);
        black_score += self.count_material(board, Color::Black);
        if self.check_two_bishops(board, Color::White) {
            white_score += 30;
        }
        if self.check_two_bishops(board, Color::Black) {
            black_score += 30;
        }
        if self.check_rooks_same_file(board, Color::White) {
            white_score += 20;
        }
        if self.check_rooks_same_file(board, Color::Black) {
            black_score += 20;
        }
        white_score += self.pawn_advancement_score(board, Color::White);
        black_score += self.pawn_advancement_score(board, Color::Black);
        let mut score: i32 = white_score - black_score;
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
        for m in moves {
            let mut score: i32 = 0;
            if *m == tt_move {
                score += 1000;
            }
            if *m == self.killer_table[ply as usize] {
                score += 100; //TODO: revisit this constant
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
                score += target_value - attacker_value + 100;
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

    fn quiesce(&mut self, board: &Board, alpha: i32, beta: i32, ply: u32) -> i32 {
        //quiesce the position
        self.nodes += 1;
        let stand_pat: i32 = self.evaluate(board);
        if stand_pat >= beta {
            return beta;
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
            let score: i32 = -self.quiesce(&new_board, -beta, -local_alpha, ply + 1);
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
        self.nodes += 1;
        if board.status() != GameStatus::Ongoing {
            match board.status() {
                GameStatus::Won => return self.min_val + (ply as i32),
                GameStatus::Drawn => return 0,
                _ => (),
            };
        }
        if depth == 0 {
            return self.quiesce(board, alpha, beta, ply);
        }
        if start_time.elapsed() > time_limit {
            return self.min_val;
        }

        // probe TT
        let mut best_score: i32 = self.min_val;
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
        let mut scores: Vec<i32> = self.score_moves(board, &moves, tt_move, ply);
        //use insertion sort to sort moves and scores
        self.sort_moves(&mut moves, &mut scores);
        //search through all moves
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
                self.killer_table[ply as usize] = m;
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
        let start_time: Instant = Instant::now();
        let time_limit: Duration = Duration::from_millis(thinking_time);
        //do iterative deepening until we run out of time
        let mut current_depth: i32 = 1;
        let mut final_move: String = String::new();
        self.nodes = 0;
        self.root_best_move = Move::from_str("a1a1").unwrap();
        while start_time.elapsed() < time_limit && current_depth < 100 {
            let score: i32 = self.negamax(board, current_depth, self.min_val, -self.min_val, 0, start_time, time_limit);
            if score.abs() != self.min_val.abs() {
                // println!("info depth {} score {}", current_depth, score);
                final_move = self.root_best_move.clone().to_string();
            }
            println!("depth {} score cp {} NPS {}k", current_depth, score, (self.nodes as f32) / (start_time.elapsed().as_secs_f32() *1000.0));
            current_depth += 1;
        }

        //check if final_move is legal
        // let mut legal_moves: Vec<Move> = Vec::new();
        // legal_moves.reserve(32);
        // board.generate_moves(|p: PieceMoves| {
        //     for m in p {
        //         legal_moves.push(m);
        //     }
        //     false
        // });
        println!("info depth {} score cp {} NPS {}k", current_depth - 1, self.root_score, (self.nodes as f32) / (start_time.elapsed().as_secs_f32() *1000.0));
        return final_move;
    }
}
