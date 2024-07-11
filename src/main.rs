mod search;

use cozy_chess::*;
use search::AlphaBetaSearcher;

fn main() {
    let mut board = Board::default();
    let mut input = String::new();
    let mut searcher = AlphaBetaSearcher::new();
    let mut btime: u64 = 0;
    let mut wtime: u64 = 0;
    let mut binc: u64 = 0;
    let mut winc: u64 = 0;
    // let mut movestogo: u64 = 0;
    // println!("The square B1 is {} when cast to usize", Square::B1 as usize);
    // println!("evaluation of startpos: {}", searcher.pesto_evaluate(&board));
    loop {
        input.clear();
        searcher.clear_threefold_repetition();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.starts_with("ucinewgame") {
            board = Board::default();
        } else if input.starts_with("uci") {
            println!("id name slinky_chess");
            println!("id author Nathan");
            println!("uciok");
        } else if input.starts_with("isready") {
            println!("readyok");
        } else if input.starts_with("position startpos moves") {
            board = Board::default();
            let moves = input.split_whitespace().skip(3);
            for m in moves {
                match util::parse_uci_move(&board, m) {
                    Ok(ucimove) => board.play(ucimove),
                    Err(e) => {
                        eprintln!("Failed to parse move: {}. Error: {:?}", m, e);
                        break;
                    }
                }
            }
        } else if input.starts_with("position startpos") {
            board = Board::default();
        } else if input.starts_with("position fen") {
            let parts: Vec<&str> = input.split_whitespace().collect();
            let fen_end = parts.iter().position(|&x| x == "moves").unwrap_or(parts.len());
            let fen = parts[2..fen_end].join(" ");
            
            match Board::from_fen(&fen, false) {
                Ok(new_board) => board = new_board,
                Err(e) => {
                    eprintln!("Failed to parse FEN: {}. Error: {:?}", fen, e);
                    continue;
                }
            }
            
            if let Some(moves_index) = parts.iter().position(|&x| x == "moves") {
                for m in parts.iter().skip(moves_index + 1) {
                    match util::parse_uci_move(&board, m) {
                        Ok(ucimove) => {
                            board.play(ucimove);
                            searcher.add_to_threefold_repetition(board.hash());
                            
                        },
                        Err(e) => {
                            eprintln!("Failed to parse move: {}. Error: {:?}", m, e);
                            break;
                        }
                    }
                }
            }
        } else if input.starts_with("go") {
            let words: Vec<&str> = input.split_whitespace().collect();
            let mut i = 0;
            let mut thinking_time: u64 = 0;
            while i < words.len() {
                match words[i] {
                    "wtime" | "btime" | "winc" | "binc" | "movetime" => {
                        if i + 1 < words.len() {
                            if let Ok(value) = words[i + 1].parse::<u64>() {
                                match words[i] {
                                    "wtime" => wtime = value,
                                    "btime" => btime = value,
                                    "winc" => winc = value,
                                    "binc" => binc = value,
                                    "movetime" => thinking_time = value,
                                    _ => unreachable!(),
                                }
                            } else {
                                eprintln!("Error parsing {}: Invalid number", words[i]);
                            }
                            i += 2;
                        } else {
                            eprintln!("Missing value for {}", words[i]);
                            i += 1;
                        }
                    },
                    // "movestogo" => {
                    //     if i + 1 < words.len() {
                    //         if let Ok(value) = words[i + 1].parse::<u64>() {
                    //             movestogo = value;
                    //         } else {
                    //             eprintln!("Error parsing movestogo: Invalid number");
                    //         }
                    //     }
                    //     i += 2;
                    // },
                    _ => i += 1,
                }
            }
            if thinking_time == 0 {
                thinking_time = if board.side_to_move() == Color::White {
                    wtime / 30 + winc / 10
                } else {
                    btime / 30 + binc / 10
                };
            }

            let best_move: String = searcher.get_best_move(&board, thinking_time);
            println!("bestmove {}", best_move);
        } else if input.starts_with("quit") {
            break;
        }
    }
}