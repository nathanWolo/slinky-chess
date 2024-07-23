use cozy_chess::*;
use crate::constants::*;
use crate::evaluation::*;
use std::str::FromStr;
use std::time::{Duration, Instant};
const TT_SIZE: usize = 1 << 22;
pub struct AlphaBetaSearcher {
    transposition_table: Vec<TTEntry>,
    root_best_move: Move,
    root_score: i32,
    min_val: i32,
    nodes: u64,
    killer_table: Vec<Move>,
    history_table: Vec<Vec<Vec<i32>>>,
    threefold_repetition: Vec<u64>, //keep a running stack of boards seen in the DFS
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
            history_table: vec![vec![vec![0; 64]; 64]; 2],
            threefold_repetition: Vec::new(),
            nodes: 0,
        }
    }
    pub fn add_to_threefold_repetition(&mut self, hash: u64) {
        self.threefold_repetition.push(hash);
    }
    pub fn clear_threefold_repetition(&mut self) {
        self.threefold_repetition = Vec::new();
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
            //idea: malus for underpromotions
            if m.promotion.is_some() {
                if m.promotion.unwrap() != Piece::Queen {
                    score -= CAPTURE_BONUS * 2;
                }
                else {
                    score += CAPTURE_BONUS;
                
                }
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
    fn piece_value(&self, piece: Piece) -> i32 {
        match piece {
            Piece::Pawn => 100,
            Piece::Knight => 320,
            Piece::Bishop => 330,
            Piece::Rook => 500,
            Piece::Queen => 900,
            Piece::King => 20000,
        }
    }
    fn see_worst_case(&self, b: &Board, m: Move) -> i32 {
        //assume piece will make move and immediately be lost for nothing
        let cap_option: Option<Piece> = b.piece_on(m.to);
        let cap_value: i32 = match cap_option {
            Some(p) => self.piece_value(p),
            None => 0,
        };
        let attacker_value: i32 = self.piece_value(b.piece_on(m.from).unwrap());
        cap_value - attacker_value
    }
    fn quiesce(&mut self, board: &Board, alpha: i32, beta: i32, ply: u32, start_time: Instant, time_limit: Duration) -> i32 {
        //quiesce the position
        self.nodes += 1;
        let stand_pat: i32 = pesto_evaluate_from_scratch(board);
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
            let worst_case: i32 = self.see_worst_case(&new_board, m);
            let at_least: i32 = stand_pat + worst_case;
            if at_least > beta {
                return beta;
            }
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

    fn pvs(&mut self, board: &Board, depth: i32, alpha: i32, beta: i32, ply:u32, start_time: Instant, time_limit: Duration, can_null: bool) -> i32 {
        self.nodes += 1;
        if board.status() != GameStatus::Ongoing {
            match board.status() {
                GameStatus::Won => return self.min_val + (ply as i32),
                GameStatus::Drawn => return 0,
                _ => (),
            };
        }
        let root: bool = ply == 0;
        //check if there is a triplet in the threefold repetition stack
        if !root {
            let mut threefold_count: i32 = 0;
            for hash in self.threefold_repetition.iter() {
                if *hash == board.hash() {
                    threefold_count += 1;
                }
            }
            if threefold_count >= 2 {
                return 0;
            }
        }


        //check extension: if in check, increase depth by 1
        let mut depth_modifier: i32 = 0;
        let in_check: bool = board.checkers().len() > 0;
        if in_check  && !root{
            depth_modifier += 1;
            // extend_checks = false;
        }

        if depth + depth_modifier <= 0 {
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
        if entry.hash == board.hash() && entry.depth >= depth && !root && !pv_node {
            match entry.node_type {
                NodeType::Exact => return entry.score,
                NodeType::LowerBound => new_alpha = alpha.max(entry.score),
                NodeType::UpperBound => new_beta = beta.min(entry.score),
            }
            if new_alpha >= new_beta {
                return entry.score;
            }
        }
        let mut can_fp: bool = false;
        //reverse futility pruning
        if !pv_node && !in_check && !root{
            let stand_pat: i32 = pesto_evaluate_from_scratch(board);
            if stand_pat - 90 * depth > beta && depth < 8{
                return stand_pat;
            }
            //null move pruning
            if stand_pat >= beta && depth > 3 && !in_check && can_null{
                let nulled_board: Board = board.clone().null_move().unwrap();
                let score: i32 = -self.pvs(&nulled_board, depth - 3, -new_beta, -new_beta + 1, ply + 1, start_time, time_limit, false);
                if score >= beta {
                    return beta;
                }
            }

            // futile pruning
            can_fp = (stand_pat + 160 * depth) < alpha && depth < 5;
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
            let is_capture: bool = self.move_is_capture(board, m);
            if can_fp && i > 4 && !is_capture {
                continue;
            }
            new_board.play(*m);
            self.threefold_repetition.push(new_board.hash());
            //extension on promotion to queen
            let mut mv_extension: i32 = 0;
            if m.promotion.is_some() {
                if m.promotion.unwrap() == Piece::Queen {
                    mv_extension += 1;
                }
            }

            let search_depth: i32 = depth + depth_modifier + mv_extension - 1;
            if i == 0 { //principal variation
                score = -self.pvs(&new_board, search_depth, -new_beta, -new_alpha, ply + 1, start_time, time_limit, can_null);
            }
            else {
                //lmr
                // let mut lmr_reduction: i32 = 0;
                // if i > 8 && !root && !in_check && depth > 2 {
                //     let history_score = self.history_table[board.side_to_move() as usize][m.from as usize][m.to as usize];
                //     let float_d = depth as f32;
                //     let float_i = i as f32;
                //     if is_capture {
                //         lmr_reduction = (0.1 + float_d.ln() * float_i.ln() * 0.3) as i32;
                //     }
                //     else if history_score > 0{
                //         lmr_reduction = (0.2 + float_d.ln() * float_i.ln() * 0.4) as i32;
                //     }
                //     else {
                //         lmr_reduction = (0.3 + float_d.ln() * float_i.ln() * 0.5) as i32;
                //     }
                // }
                score = -self.pvs(&new_board, search_depth, -new_alpha - 1, -new_alpha, ply + 1, start_time, time_limit, can_null);
                if new_alpha < score && score < new_beta {
                    score = -self.pvs(&new_board, search_depth, -new_beta, -score, ply + 1, start_time, time_limit, can_null);
                }
            }
            self.threefold_repetition.pop();
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
                //history gravity
                //decrease history for all prior moves that didnt cause the cutoff
                //if the beta cutoff move was not a capture
                if !self.move_is_capture(board, m) {
                    for j in 0..i {
                        self.history_table[board.side_to_move() as usize][moves[j].from as usize][moves[j].to as usize] -= 1;
                    }
                }
                break;
            }
            // else {
            //     //history gravity, decrease all history values of moves that dont cause a beta cutoff
            //     self.history_table[board.side_to_move() as usize][m.from as usize][m.to as usize] -= 1;
            // }
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

    pub fn get_best_move(&mut self, board: &Board, time_remaining: u64) -> String {
        let start_time: Instant = Instant::now();
        let hard_limit: Duration = Duration::from_millis(time_remaining/10);
        let soft_limit: Duration = Duration::from_millis(time_remaining/40);
        //do iterative deepening until we run out of time
        let mut current_depth: i32 = 1;
        let final_move: String;
        self.nodes = 0;
        self.root_best_move = Move::from_str("a1a1").unwrap();
        //clear history table
        self.history_table = vec![vec![vec![0; 64]; 64]; 2];
        // self.killer_table = vec![Move::from_str("a1a1").unwrap(); 128];

        let mut aspiration_window: i32 = 15;
        let mut alpha: i32 = -99999999;
        let mut beta: i32 = 99999999;

        while start_time.elapsed() < soft_limit && current_depth < 100 {
            let score: i32 = self.pvs(board, current_depth, alpha, beta, 0, start_time, hard_limit, true);
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
