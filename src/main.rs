use cozy_chess::*;
// use rand::Rng;
fn main() {
    // Simple implementation of the UCI protocol
    //read from stdin and write to stdout. If a move is requested, generate a random move
    let mut board = Board::default();
    let mut input = String::new();
//    let mut btime: i32 = 0;
//    let mut wtime: i32 = 0;
//     let mut binc: i32 = 0;
//     let mut winc: i32 = 0;
//     let mut movestogo: i32 = 0;
    loop {
        input.clear();
        std::io::stdin().read_line(&mut input).unwrap();
        if input.starts_with("ucinewgame") {
            // Reset the board
            board = Board::default();
        }
        else if input.starts_with("uci") {
            println!("id name slinky_chess");
            println!("id author Nathan");
            println!("uciok");
        } else if input.starts_with("isready") {
            println!("readyok");
        }
        else if input.starts_with("position startpos moves") {
            // Reset the board
            board = Board::default();
            input = input.replace("\n", "").replace("\r", "");
            let moves = input.split(" ").skip(3);
            
            for m in moves {
                match util::parse_uci_move(&board, m) {
                    Ok(ucimove) => board.play(ucimove),
                    Err(e) => {
                        eprintln!("Failed to parse move: {}. Error: {:?}", m, e);
                        break;
                    }
                }
            }
        }
        else if input.starts_with("position startpos") {
            // Reset the board
            board = Board::default();
        }
         else if input.starts_with("go") {
            // Parse time controls
        //    let mut tokens = input.split(" ");
        //    while let Some(token) = tokens.next() {
        //        match token {
        //            "btime" => btime = tokens.next().unwrap().parse().unwrap(),
        //            "wtime" => wtime = tokens.next().unwrap().parse().unwrap(),
        //            "binc" => binc = tokens.next().unwrap().parse().unwrap(),
        //            "winc" => winc = tokens.next().unwrap().parse().unwrap(),
        //            // "movestogo" => movestogo = tokens.next().unwrap().parse().unwrap(),
        //            _ => {}
        //        }
        //     }
            let best_move = get_best_move(&board, 3, board.side_to_move());
            println!("bestmove {}", best_move);

        }
        else if input.starts_with("quit") {
            break;
        }
    }
}

// fn get_random_move(board: &Board) -> String {
//     let mut move_list = Vec::new();
//     board.generate_moves(|moves| {
//         // Unpack dense move set into move list
//         move_list.extend(moves);
//         false
//     });
//     let random_index = rand::thread_rng().gen_range(0..move_list.len());
//     return move_list[random_index].to_string()
// }

fn count_material(board: &Board, color: Color ) -> i32 {
    let mut material: i32 = 0;
    //create static array of each piece type
    let pieces: [Piece; 5] = [Piece::Pawn, Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen];
    let values: [i32; 5] = [1, 3, 3, 5, 9];
    for (i, piece) in pieces.iter().enumerate() {
        let count: i32 = board.colored_pieces(color, *piece).len() as i32;
        material += count * values[i];
    }
    return material;
}

fn evaluate(board: &Board, stm: Color) -> i32 {
    let mut score: i32;
    let white_material = count_material(board, Color::White);
    let black_material = count_material(board, Color::Black);
    score = white_material - black_material;
    if stm == Color::Black {
        score = -score;
    }
    return score;
}

fn negamax(board: &Board, depth: i32, alpha: i32, beta: i32, color: Color) -> i32 {
    let status: GameStatus = board.status();
    //match on status for draw or checkmate
    if status != GameStatus::Ongoing {
        match status {
            GameStatus::Won => {
                return -999;
            }
            GameStatus::Drawn => return 0,
            _ => {}
        }
    }
    if depth == 0 {
        return evaluate(board, color);
    }
    let mut best_score = -999;
    let mut new_alpha = alpha;
    board.generate_moves(|moves: PieceMoves| {
        for m in moves {
            let mut new_board: Board = board.clone();
            new_board.play(m);
            let score: i32 = -negamax(&new_board, depth - 1, -beta, -new_alpha, !color);
            best_score = best_score.max(score);
            new_alpha = new_alpha.max(score);
            if new_alpha >= beta {
                break;
            }
        }
        false
    });
    return best_score;
}

fn get_best_move(board: &Board, depth: i32, color: Color) -> String {
    let mut best_move = String::new();
    let mut best_score = -1000;
    board.generate_moves(|moves| {
        for m in moves {
            let mut new_board = board.clone();
            new_board.play(m);
            let score = -negamax(&new_board, depth - 1, -1000, 1000, !color);
            if score > best_score {
                best_score = score;
                best_move = m.to_string();
            }
        }
        false
    });
    return best_move;
}