// use cozy_chess::*;
// use crate::evaluation::*;
// use crate::constants::*;
// pub struct LinearAccumulator {
//     pub board: Board,
//     pub eval: i32, //stm relative
// }

// //this is a struct for iteratively updating the evaluation of a board
// //using piece square tables
// impl LinearAccumulator {
//     pub fn new(board: Board) -> LinearAccumulator {
//         LinearAccumulator {
//             board,
//             eval: 0,
//         }
//     }

//     pub fn update(&mut self, mv: Move) {
//         //update the evaluation based on the move
//         //will involve subtracting the old piece square value
//         //and adding the new piece square value
//         //and then dealing with captures and promotions

//         //first, get the piece that is moving
//         let moving_piece: Piece = self.board.piece_on(mv.from).unwrap();
//         let from_square: Square = mv.from;
//         let to_square: Square = mv.to;
//         //check if there is a capture
//         let capture: Option<Piece> = self.board.piece_on(to_square);
//         if capture.is_some() {
//             //subtract the value of the captured piece
//             let captured_piece: Piece = capture.unwrap();
//             let captured_piece_value: i32 = piece_value(captured_piece);
//             self.eval -= captured_piece_value;
//         }
//     }
// }