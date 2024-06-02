use cozy_chess::*;
use rand::Rng;
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
            
            // // Generate a random move
            // let mut move_list = Vec::new();
            // board.generate_moves(|moves| {
            //     // Unpack dense move set into move list
            //     move_list.extend(moves);
            //     false
            // });
            // let random_index = rng.gen_range(0..move_list.len());
            let best_move = get_best_move(&board);
            println!("bestmove {}", best_move);

        }
        else if input.starts_with("quit") {
            break;
        }
    }
}

fn get_best_move(board: &Board) -> String {
    let mut move_list = Vec::new();
    board.generate_moves(|moves| {
        // Unpack dense move set into move list
        move_list.extend(moves);
        false
    });
    let random_index = rand::thread_rng().gen_range(0..move_list.len());
    move_list[random_index].to_string()
}