use arrayvec::ArrayVec;
use cozy_chess::*;
use crate::constants::*;
use crate::evaluation::*;
use std::str::FromStr;
use std::time::{Duration, Instant};
const TT_SIZE: usize = 1 << 24;
const HISTORY_MAX: i32 = 30000;
pub struct AlphaBetaSearcher {
    transposition_table: Vec<TTEntry>,
    root_best_move: Move,
    root_score: i32,
    min_val: i32,
    nodes: u64,
    killer_table: [[Move; 2]; 128],
    history_table: [[[i32; 64]; 64]; 2],
    threefold_repetition: Vec<u64>, //keep a running stack of boards seen in the DFS
}
#[derive(Clone, Copy)]
struct TTEntry { // 16 bytes total
    hash: u64, //4 bytes
    depth: i32, //2 bytes
    score: i32, //2 bytes
    best_move: Move, // 8 bytes
    node_type: NodeType,
}
#[derive(Clone, Copy)]
enum NodeType {
    Exact,
    LowerBound,
    UpperBound,
}

fn dummy_move() -> Move {
    Move::from_str("a1a1").unwrap()
}

impl AlphaBetaSearcher {
    pub fn new() -> Self {
        AlphaBetaSearcher {
            root_best_move: dummy_move(),
            root_score: 0,
            min_val: - (1 << 30),
            transposition_table: vec![TTEntry {
                hash: 0,
                depth: 0,
                score: 0,
                best_move: dummy_move(),
                node_type: NodeType::Exact,
            }; TT_SIZE],
            killer_table: [[dummy_move(); 2]; 128],
            history_table: [[[0; 64]; 64]; 2],
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
        if board.piece_on(m.to).is_some() {
            return true;
        }
        //en passant: pawn diagonal onto the empty EP square
        if let Some(ep_file) = board.en_passant() {
            if m.to.file() == ep_file
                && m.from.file() != m.to.file()
                && board.piece_on(m.from) == Some(Piece::Pawn)
            {
                return true;
            }
        }
        false
    }

    fn captured_piece_value(&self, board: &Board, m: Move) -> i32 {
        if let Some(piece) = board.piece_on(m.to) {
            return self.piece_value(piece);
        }
        if self.move_is_capture(board, &m) {
            return self.piece_value(Piece::Pawn);
        }
        0
    }

    fn mvv_lva_value(&self, piece: Piece) -> i32 {
        match piece {
            Piece::Pawn => 1,
            Piece::Knight => 3,
            Piece::Bishop => 3,
            Piece::Rook => 5,
            Piece::Queen => 9,
            _ => 0,
        }
    }

    fn score_moves(&self, _board: &Board, moves: &ArrayVec<[Move; 256]>, tt_move: Move, ply: u32) -> ArrayVec<[i32; 256]> {
        //take in a board and a list of moves and return a list of scores for each move
        let mut scores = ArrayVec::<[i32; 256]>::new();
        for m in moves {
            let mut score: i32 = 0;
            if *m == tt_move {
                score += TT_BONUS;
            }
            // Most valuable victim - least valuable attacker
            if self.move_is_capture(_board, m) {
                let attacker: Piece = _board.piece_on(m.from).unwrap();
                let target_value: i32 = match _board.piece_on(m.to) {
                    Some(piece) => self.mvv_lva_value(piece),
                    None => self.mvv_lva_value(Piece::Pawn), //en passant
                };
                score += target_value * 20 - self.mvv_lva_value(attacker) + CAPTURE_BONUS;
            }
            else if *m == self.killer_table[ply as usize][0] {
                score += KILLER_BONUS;
            }
            else if *m == self.killer_table[ply as usize][1] {
                score += KILLER_BONUS / 2;
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

    fn sort_moves(&self, moves: &mut ArrayVec<[Move; 256]>, scores: &mut ArrayVec<[i32; 256]>) {
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
        let cap_value: i32 = self.captured_piece_value(b, m);
        let attacker_value: i32 = self.piece_value(b.piece_on(m.from).unwrap());
        let mut value: i32 = cap_value - attacker_value;
        if let Some(promo) = m.promotion {
            value += self.piece_value(promo) - self.piece_value(Piece::Pawn);
        }
        value
    }

    fn age_history(&mut self) {
        for color in self.history_table.iter_mut() {
            for from in color.iter_mut() {
                for hist in from.iter_mut() {
                    *hist /= 2;
                }
            }
        }
    }

    fn add_history(&mut self, stm: usize, from: usize, to: usize, delta: i32) {
        let hist: &mut i32 = &mut self.history_table[stm][from][to];
        *hist += delta;
        if *hist > HISTORY_MAX {
            *hist = HISTORY_MAX;
        } else if *hist < -HISTORY_MAX {
            *hist = -HISTORY_MAX;
        }
    }

    fn store_killer(&mut self, ply: u32, m: Move) {
        let ply: usize = ply as usize;
        if self.killer_table[ply][0] != m {
            self.killer_table[ply][1] = self.killer_table[ply][0];
            self.killer_table[ply][0] = m;
        }
    }

    fn probe_tt(&self, board: &Board) -> Option<TTEntry> {
        let entry: TTEntry = self.transposition_table[board.hash() as usize % TT_SIZE];
        if entry.hash == board.hash() {
            Some(entry)
        } else {
            None
        }
    }

    fn store_tt(&mut self, board: &Board, depth: i32, score: i32, best_move: Move, node_type: NodeType) {
        if score.abs() == self.min_val.abs() {
            return;
        }
        let idx: usize = board.hash() as usize % TT_SIZE;
        let old: TTEntry = self.transposition_table[idx];
        //keep a deeper entry for the same position (qsearch must not clobber it)
        if old.hash == board.hash() && old.depth > depth {
            return;
        }
        self.transposition_table[idx] = TTEntry {
            hash: board.hash(),
            depth,
            score,
            best_move,
            node_type,
        };
    }

    fn has_non_pawn_material(&self, board: &Board) -> bool {
        let us: BitBoard = board.colors(board.side_to_move());
        let pieces: BitBoard = us & !board.pieces(Piece::Pawn) & !board.pieces(Piece::King);
        !pieces.is_empty()
    }

    fn quiesce(&mut self, board: &Board, alpha: i32, beta: i32, ply: u32, start_time: Instant, time_limit: Duration) -> i32 {
        self.nodes += 1;
        if start_time.elapsed() > time_limit {
            return self.min_val;
        }
        if board.halfmove_clock() >= 100 {
            return 0;
        }

        let in_check: bool = !board.checkers().is_empty();
        let mut local_alpha: i32 = alpha;
        let stand_pat: i32 = pesto_evaluate_from_scratch(board);

        if let Some(entry) = self.probe_tt(board) {
            match entry.node_type {
                NodeType::Exact => return entry.score,
                NodeType::LowerBound if entry.score >= beta => return entry.score,
                NodeType::UpperBound if entry.score <= alpha => return entry.score,
                _ => (),
            }
        }

        if !in_check {
            if stand_pat >= beta {
                return beta;
            }
            local_alpha = local_alpha.max(stand_pat);
        }

        let mut moves = ArrayVec::<[Move; 256]>::new();
        board.generate_moves(|p: PieceMoves| {
            for m in p {
                if in_check || self.move_is_capture(board, &m) || m.promotion.is_some() {
                    moves.push(m);
                }
            }
            false
        });
        if in_check && moves.is_empty() {
            return self.min_val + (ply as i32);
        }

        let tt_move: Move = self.probe_tt(board).map(|e| e.best_move).unwrap_or_else(dummy_move);
        let mut scores: ArrayVec<[i32; 256]> = self.score_moves(board, &moves, tt_move, ply);
        self.sort_moves(&mut moves, &mut scores);
        let mut best_move: Move = tt_move;
        let original_alpha: i32 = alpha;
        for m in moves {
            if !in_check {
                let worst_case: i32 = self.see_worst_case(board, m);
                let at_least: i32 = stand_pat + worst_case;
                if at_least > beta {
                    return beta;
                }
                let optimistic: i32 = stand_pat + self.captured_piece_value(board, m)
                    + m.promotion.map(|p| self.piece_value(p) - self.piece_value(Piece::Pawn)).unwrap_or(0)
                    + 200;
                if optimistic < local_alpha {
                    continue;
                }
            }
            let mut new_board: Board = board.clone();
            new_board.play_unchecked(m);
            let score: i32 = -self.quiesce(&new_board, -beta, -local_alpha, ply + 1, start_time, time_limit);
            if score >= beta {
                self.store_tt(board, 0, score, m, NodeType::LowerBound);
                return beta;
            }
            if score > local_alpha {
                local_alpha = score;
                best_move = m;
            }
        }
        let node_type: NodeType = if local_alpha <= original_alpha {
            NodeType::UpperBound
        } else {
            NodeType::Exact
        };
        self.store_tt(board, 0, local_alpha, best_move, node_type);
        local_alpha
    }

    fn pvs(&mut self, board: &Board, mut depth: i32, alpha: i32, beta: i32, ply:u32, start_time: Instant, time_limit: Duration, can_null: bool) -> i32 {
        self.nodes += 1;
        if board.status() != GameStatus::Ongoing {
            match board.status() {
                GameStatus::Won => return self.min_val + (ply as i32),
                GameStatus::Drawn => return 0,
                _ => (),
            };
        }
        let root: bool = ply == 0;
        if !root && board.halfmove_clock() >= 100 {
            return 0;
        }
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
        let in_check: bool = !board.checkers().is_empty();
        if in_check  && !root{
            depth_modifier += 1;
        }

        if start_time.elapsed() > time_limit {
            return self.min_val;
        }
        let pv_node: bool = beta - alpha > 1;
        // probe TT
        let mut best_score: i32 = self.min_val;
        let mut new_alpha: i32 = alpha;
        let mut new_beta: i32 = beta;
        let tt_entry: Option<TTEntry> = self.probe_tt(board);
        let tt_move: Move = tt_entry.map(|e| e.best_move).unwrap_or_else(dummy_move);
        if let Some(entry) = tt_entry {
            if entry.depth >= depth && !root && !pv_node {
                match entry.node_type {
                    NodeType::Exact => return entry.score,
                    NodeType::LowerBound => new_alpha = alpha.max(entry.score),
                    NodeType::UpperBound => new_beta = beta.min(entry.score),
                }
                if new_alpha >= new_beta {
                    return entry.score;
                }
            }
        }

        //internal iterative reduction: no TT move means this node is cheap to reduce
        if !pv_node && !in_check && !root && tt_entry.is_none() && depth >= 4 {
            depth -= 1;
        }

        if depth + depth_modifier <= 0 {
            return self.quiesce(board, alpha, beta, ply, start_time, time_limit);
        }
        let mut can_fp: bool = false;
        //reverse futility pruning
        if !pv_node && !in_check && !root{
            let stand_pat: i32 = pesto_evaluate_from_scratch(board);
            if stand_pat - 90 * depth > beta && depth < 8{
                return stand_pat;
            }
            //razoring: if even a large margin cannot raise alpha, drop into qsearch
            if depth <= 2 && stand_pat + 150 * depth < alpha {
                let qscore: i32 = self.quiesce(board, alpha, beta, ply, start_time, time_limit);
                if qscore < alpha {
                    return qscore;
                }
            }
            //null move pruning
            if stand_pat >= beta && depth > 3 && can_null && self.has_non_pawn_material(board) {
                if let Some(nulled_board) = board.null_move() {
                    let r: i32 = 3 + (depth - 4) / 4;
                    let score: i32 = -self.pvs(&nulled_board, depth - r, -new_beta, -new_beta + 1, ply + 1, start_time, time_limit, false);
                    if score >= beta {
                        return beta;
                    }
                }
            }

            // futile pruning
            can_fp = (stand_pat + 160 * depth) < alpha && depth < 5;
        }

        //generate all moves and store them in a vector
        let mut moves = ArrayVec::<[Move; 256]>::new();
        board.generate_moves(|p: PieceMoves| {
            for m in p {
                moves.push(m);
            }
            false
        });
        let mut scores: ArrayVec<[i32; 256]> = self.score_moves(board, &moves, tt_move, ply);
        self.sort_moves(&mut moves, &mut scores);
        let mut score: i32;
        let mut node_best_move: Move = tt_move;
        let original_alpha: i32 = alpha;

        for (i, m) in moves.iter().enumerate() {
            let is_capture: bool = self.move_is_capture(board, m);
            let is_quiet: bool = !is_capture && m.promotion.is_none();
            if can_fp && i > 4 && is_quiet {
                continue;
            }
            //late move pruning: skip remaining quiet moves at low depth
            if !pv_node && !in_check && is_quiet && depth <= 4 && i >= (3 + (depth * depth) as usize) {
                continue;
            }
            let mut new_board = board.clone();
            new_board.play_unchecked(*m);
            self.threefold_repetition.push(new_board.hash());
            //extension on promotion to queen
            let mut mv_extension: i32 = 0;
            if m.promotion.is_some() {
                if m.promotion.unwrap() == Piece::Queen {
                    mv_extension += 1;
                }
            }

            let search_depth: i32 = depth + depth_modifier + mv_extension - 1;
            let gives_check: bool = !new_board.checkers().is_empty();
            //lmr
            let mut lmr_depth: i32 = search_depth;
            if i > 7 && depth > 2 && !gives_check && !in_check && is_quiet && !pv_node {
                lmr_depth -= 2;
            }
            else if i > 7 && depth > 2 && !gives_check && !in_check {
                lmr_depth -= 1;
            }
            if i == 0 { //principal variation
                score = -self.pvs(&new_board, search_depth, -new_beta, -new_alpha, ply + 1, start_time, time_limit, can_null);
            }
            else {
                score = -self.pvs(&new_board, lmr_depth, -new_alpha - 1, -new_alpha, ply + 1, start_time, time_limit, can_null);
                if new_alpha < score { 
                    if lmr_depth < search_depth { //if it was an lmr node
                        score = -self.pvs(&new_board, search_depth, -new_alpha - 1, -new_alpha, ply + 1, start_time, time_limit, can_null);
                    }
                    //full re-search
                    if new_alpha < score {
                        score = -self.pvs(&new_board, search_depth, -new_beta, -new_alpha, ply + 1, start_time, time_limit, can_null);
                    }
                }
            }
            self.threefold_repetition.pop();
            if score > best_score {
                best_score = score;
                node_best_move = *m;
                if (ply == 0) && (score.abs() != self.min_val.abs()) {
                    self.root_best_move = *m;
                    self.root_score = score;
                }
            }
            new_alpha = new_alpha.max(score);
            if new_alpha >= new_beta {
                if is_quiet {
                    self.store_killer(ply, *m);
                    let bonus: i32 = depth * depth;
                    self.add_history(board.side_to_move() as usize, m.from as usize, m.to as usize, bonus);
                    for j in 0..i {
                        if !self.move_is_capture(board, &moves[j]) && moves[j].promotion.is_none() {
                            self.add_history(board.side_to_move() as usize, moves[j].from as usize, moves[j].to as usize, -bonus);
                        }
                    }
                }
                break;
            }
        }
        let node_type: NodeType = if best_score <= original_alpha {
            NodeType::UpperBound
        } else if best_score >= beta {
            NodeType::LowerBound
        } else {
            NodeType::Exact
        };
        self.store_tt(board, depth, best_score, node_best_move, node_type);
        
        best_score
    }

    pub fn get_best_move(&mut self, board: &Board, time_remaining: u64, increment: u64, is_movetime: bool) -> String {
        let start_time: Instant = Instant::now();
        let (hard_ms, soft_ms): (u64, u64) = if is_movetime {
            (time_remaining.saturating_sub(8).max(1), (time_remaining * 4 / 5).max(1))
        } else {
            (time_remaining / 10 + increment, time_remaining / 40 + increment / 2)
        };
        let hard_limit: Duration = Duration::from_millis(hard_ms.max(1));
        let soft_limit: Duration = Duration::from_millis(soft_ms.max(1).min(hard_ms.max(1)));
        //do iterative deepening until we run out of time
        let mut current_depth: i32 = 1;
        let final_move: String;
        self.nodes = 0;
        self.root_best_move = dummy_move();
        //age history so ordering persists across moves without overflowing
        self.age_history();

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
        if !board.is_legal(self.root_best_move) {
            panic!("Illegal move {} in position {}. Searched to depth {} with root_best_move {}", final_move, board, current_depth - 1, self.root_best_move);
        }
        println!("info depth {} score cp {} NPS {}k", current_depth - 1, self.root_score, (self.nodes as f32) / (start_time.elapsed().as_secs_f32() *1000.0));
        return final_move;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_board(fen: &str) -> Board {
        Board::from_fen(fen, false).expect(fen)
    }

    #[test]
    fn en_passant_counts_as_capture() {
        let board = parse_board("4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1");
        let searcher = AlphaBetaSearcher::new();
        let ep = Move {
            from: Square::E5,
            to: Square::D6,
            promotion: None,
        };
        assert!(searcher.move_is_capture(&board, &ep));
        assert_eq!(searcher.captured_piece_value(&board, ep), 100);
    }

    #[test]
    fn finds_mate_in_one() {
        let board = parse_board("6k1/5ppp/8/8/8/8/8/4R1K1 w - - 0 1");
        let mut searcher = AlphaBetaSearcher::new();
        let best = searcher.get_best_move(&board, 250, 0, true);
        assert_eq!(best, "e1e8");
    }

    #[test]
    fn startpos_returns_legal_move() {
        let board = Board::default();
        let mut searcher = AlphaBetaSearcher::new();
        let best = searcher.get_best_move(&board, 200, 0, true);
        let parsed = util::parse_uci_move(&board, &best).expect("uci move");
        assert!(board.is_legal(parsed));
    }
}
