use cozy_chess::*;
use crate::constants::*;
pub fn pesto_evaluate_from_scratch(board: &Board) -> i32 {
    let mut white_mg:i32 = 0;
    let mut black_mg:i32 = 0;
    let mut white_eg:i32 = 0;
    let mut black_eg:i32 = 0;
    let mut mg_phase: i32 = 0;
    let mut score: i32;
    let white: BitBoard = board.colors(Color::White);
    let black: BitBoard = board.colors(Color::Black);
    //bishop pair bonus
    let bishops: BitBoard = board.pieces(Piece::Bishop);
    if (bishops & white).len() > 1 {
        white_mg += BISHOP_PAIR_MG;
        white_eg += BISHOP_PAIR_EG;
    }
    if (bishops & black).len() > 1 {
        black_mg += BISHOP_PAIR_MG;
        black_eg += BISHOP_PAIR_EG;
    }
    for square in white.iter() {
        let piece: Piece = board.piece_on(square).unwrap(); 
        white_mg += get_square_score_mg(square, Color::White, piece);
        white_eg += get_square_score_eg(square, Color::White, piece);               
        //control mg vs eg phase
        mg_phase += piece_phase(piece);
        if piece == Piece::Rook {
            if has_open_file(board, square, Color::White) {
                white_mg += ROOK_OPEN_FILE_MG;
                white_eg += ROOK_OPEN_FILE_EG;
            }
            else if has_semi_open_file(board, square, Color::White) {
                white_mg += ROOK_SEMI_OPEN_FILE_MG;
                white_eg += ROOK_SEMI_OPEN_FILE_EG;
            }
        }
        else if piece == Piece::Pawn {
            if pawn_is_doubled(board, square, Color::White) {
                white_mg += DOUBLED_PAWNS_MG;
                white_eg += DOUBLED_PAWNS_EG;
            }
        }
    }
    for square in black.iter() {
        let piece: Piece = board.piece_on(square).unwrap();
        black_mg += get_square_score_mg(square, Color::Black, piece);
        black_eg += get_square_score_eg(square, Color::Black, piece);
        //control mg vs eg phase
        mg_phase += piece_phase(piece);
        if piece == Piece::Rook {
            if has_open_file(board, square, Color::Black) {
                black_mg += ROOK_OPEN_FILE_MG;
                black_eg += ROOK_OPEN_FILE_EG;
            }
            else if has_semi_open_file(board, square, Color::Black) {
                black_mg += ROOK_SEMI_OPEN_FILE_MG;
                black_eg += ROOK_SEMI_OPEN_FILE_EG;
            }
        }
        else if piece == Piece::Pawn {
            if pawn_is_doubled(board, square, Color::Black) {
                black_mg += DOUBLED_PAWNS_MG;
                black_eg += DOUBLED_PAWNS_EG;
            }
        }
    }
    if board.side_to_move() == Color::White {
        white_mg += TEMPO_BONUS;
    }
    else {
        black_mg += TEMPO_BONUS;
    }
    let mg: i32 = white_mg - black_mg;
    let eg: i32 = white_eg - black_eg;
    mg_phase = mg_phase.min(24);
    let eg_phase: i32 = 24 - mg_phase;
    score = (mg * mg_phase + eg * eg_phase)/ 24;
    if board.side_to_move() == Color::Black {
        score = -score;
    }
    score
}

pub fn has_open_file(board: &Board, square: Square, side: Color) -> bool {
    //check if the piece on this square has access to an open file in front of it
    //this is used for rooks
    let file: BitBoard = square.file().bitboard();
    let other_side: Color = match side {
        Color::White => Color::Black,
        Color::Black => Color::White,
    };
    //check that there are no enemy pawns on the file
    let enemy_pawns: BitBoard = board.colored_pieces(other_side, Piece::Pawn);
    let friendly_pawns: BitBoard = board.colored_pieces(side, Piece::Pawn);
    return (file & friendly_pawns & enemy_pawns).is_empty();
}

pub fn has_semi_open_file(board: &Board, square: Square, side: Color) -> bool {
    //check if the piece on this square has access to a semi open file in front of it
    //this is used for rooks
    let file: BitBoard = square.file().bitboard();
    let friendly_pawns: BitBoard = board.colored_pieces(side, Piece::Pawn);
    return (file & friendly_pawns).is_empty();
}

pub fn pawn_is_doubled(board: &Board, square: Square, side: Color) -> bool {
    //check if the pawn on this square is doubled
    let file: BitBoard = square.file().bitboard();
    let friendly_pawns: BitBoard = board.colored_pieces(side, Piece::Pawn);
    return (file & friendly_pawns).len() > 1;
}

pub fn get_square_score_mg(square: Square, side: Color, piece: Piece) -> i32 {
    //want "relative" to other side, since A8 is first in the array
    let other_side: Color = match side {
        Color::White => Color::Black,
        Color::Black => Color::White,
    };
    let rel_square: usize = square.relative_to(other_side) as usize;
    match piece {
        Piece::Pawn => MG_PAWN_TABLE[rel_square] + MG_PAWN_MATERIAL,
        Piece::Knight => MG_KNIGHT_TABLE[rel_square] + MG_KNIGHT_MATERIAL,
        Piece::Bishop => MG_BISHOP_TABLE[rel_square] + MG_BISHOP_MATERIAL,
        Piece::Rook => MG_ROOK_TABLE[rel_square] + MG_ROOK_MATERIAL,
        Piece::Queen => MG_QUEEN_TABLE[rel_square] + MG_QUEEN_MATERIAL,
        Piece::King => MG_KING_TABLE[rel_square] + MG_KING_MATERIAL,
    }
}

pub fn get_square_score_eg(square: Square, side: Color, piece: Piece) -> i32 {
    //want "relative" to other side, since A8 is first in the array
    let other_side: Color = match side {
        Color::White => Color::Black,
        Color::Black => Color::White,
    };
    let rel_square: usize = square.relative_to(other_side) as usize;
    match piece {
        Piece::Pawn => EG_PAWN_TABLE[rel_square] + EG_PAWN_MATERIAL,
        Piece::Knight => EG_KNIGHT_TABLE[rel_square] + EG_KNIGHT_MATERIAL,
        Piece::Bishop => EG_BISHOP_TABLE[rel_square] + EG_BISHOP_MATERIAL,
        Piece::Rook => EG_ROOK_TABLE[rel_square] + EG_ROOK_MATERIAL,
        Piece::Queen => EG_QUEEN_TABLE[rel_square] + EG_QUEEN_MATERIAL,
        Piece::King => EG_KING_TABLE[rel_square] + EG_KING_MATERIAL,
    }
}

pub fn piece_phase(piece: Piece) -> i32 {
    match piece {
        Piece::Pawn => 0,
        Piece::Knight => 1,
        Piece::Bishop => 1,
        Piece::Rook => 2,
        Piece::Queen => 4,
        _ => 0,
    }
}