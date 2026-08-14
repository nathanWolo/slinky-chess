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
            if rook_on_seventh(square, Color::White, board.king(Color::Black)) {
                white_mg += ROOK_ON_SEVENTH_MG;
                white_eg += ROOK_ON_SEVENTH_EG;
            }
        }
        else if piece == Piece::Pawn {
            if pawn_is_doubled(board, square, Color::White) {
                white_mg += DOUBLED_PAWNS_MG;
                white_eg += DOUBLED_PAWNS_EG;
            }
            if pawn_is_isolated(board, square, Color::White) {
                white_mg += ISOLATED_PAWN_MG;
                white_eg += ISOLATED_PAWN_EG;
            }
            if pawn_defends_friend(board, square, Color::White) {
                white_mg += PAWN_DEFENDS_FRIEND_MG;
                white_eg += PAWN_DEFENDS_FRIEND_EG;
            
            }
            if pawn_is_passed(board, square, Color::White) {
                white_mg += PASSED_PAWN_TABLE_MG[square.relative_to(Color::Black) as usize];
                white_eg += PASSED_PAWN_TABLE_EG[square.relative_to(Color::Black) as usize];
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
            if rook_on_seventh(square, Color::Black, board.king(Color::White)) {
                black_mg += ROOK_ON_SEVENTH_MG;
                black_eg += ROOK_ON_SEVENTH_EG;
            }
        }
        else if piece == Piece::Pawn {
            if pawn_is_doubled(board, square, Color::Black) {
                black_mg += DOUBLED_PAWNS_MG;
                black_eg += DOUBLED_PAWNS_EG;
            }
            if pawn_is_isolated(board, square, Color::Black) {
                black_mg += ISOLATED_PAWN_MG;
                black_eg += ISOLATED_PAWN_EG;
            }
            if pawn_defends_friend(board, square, Color::Black) {
                black_mg += PAWN_DEFENDS_FRIEND_MG;
                black_eg += PAWN_DEFENDS_FRIEND_EG;
            }
            if pawn_is_passed(board, square, Color::Black) {
                black_mg += PASSED_PAWN_TABLE_MG[square as usize];
                black_eg += PASSED_PAWN_TABLE_EG[square as usize];
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
    //open file: no pawns of either color on this file
    let file: BitBoard = square.file().bitboard();
    let other_side: Color = match side {
        Color::White => Color::Black,
        Color::Black => Color::White,
    };
    let enemy_pawns: BitBoard = board.colored_pieces(other_side, Piece::Pawn);
    let friendly_pawns: BitBoard = board.colored_pieces(side, Piece::Pawn);
    return (file & (friendly_pawns | enemy_pawns)).is_empty();
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

pub fn pawn_is_isolated(board: &Board, square: Square, side: Color) -> bool {
    let friendly_pawns: BitBoard = board.colored_pieces(side, Piece::Pawn);
    return (square.file().adjacent() & friendly_pawns).is_empty();
}

pub fn rook_on_seventh(square: Square, side: Color, enemy_king: Square) -> bool {
    match side {
        Color::White => square.rank() == Rank::Seventh && enemy_king.rank() == Rank::Eighth,
        Color::Black => square.rank() == Rank::Second && enemy_king.rank() == Rank::First,
    }
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

pub fn pawn_defends_friend(board: &Board, square: Square, side: Color) -> bool {
    //check if a pawn on the passed square defends a friendly piece
    let friendly_pieces: BitBoard = board.colors(side);
    let pawn_attacks: BitBoard = get_pawn_attacks(square, side);
    return !(pawn_attacks & friendly_pieces).is_empty();
}

pub fn pawn_is_passed(board: &Board, square: Square, side: Color)-> bool {
    //check if the squares file and adjacent files are empty of enemy pawns
    let mut file: BitBoard = square.file().bitboard() | square.file().adjacent();
    let other_side: Color = match side {
        Color::White => Color::Black,
        Color::Black => Color::White,
    };
    //restrict the file bitboard to only squares ahead of the pawn
    if side == Color::White {
        let square_idx: usize = square as usize;
        let destination_square_idx: usize = 56 + (square_idx % 8);
        file &= get_between_rays(square, Square::index(destination_square_idx));
    }
    else {
        let square_idx: usize = square as usize;
        let destination_square_idx: usize = square_idx % 8;
        file &= get_between_rays(square, Square::index(destination_square_idx));
    }

    let enemy_pawns: BitBoard = board.colored_pieces(other_side, Piece::Pawn);
    return (file & enemy_pawns).is_empty();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_board(fen: &str) -> Board {
        Board::from_fen(fen, false).expect(fen)
    }

    #[test]
    fn open_file_requires_no_pawns_on_file() {
        // white rook on a1, pawns on both a-files would previously still count as open
        // because friendly & enemy pawn bitboards can never overlap
        let closed = parse_board("4k3/p7/8/8/8/8/P7/R3K3 w - - 0 1");
        assert!(!has_open_file(&closed, Square::A1, Color::White));
        assert!(!has_semi_open_file(&closed, Square::A1, Color::White));

        let semi = parse_board("4k3/p7/8/8/8/8/8/R3K3 w - - 0 1");
        assert!(!has_open_file(&semi, Square::A1, Color::White));
        assert!(has_semi_open_file(&semi, Square::A1, Color::White));

        let open = parse_board("4k3/8/8/8/8/8/8/R3K3 w - - 0 1");
        assert!(has_open_file(&open, Square::A1, Color::White));
    }

    #[test]
    fn isolated_pawn_has_no_friendly_neighbors() {
        let board = parse_board("4k3/8/8/8/8/8/P1P5/4K3 w - - 0 1");
        assert!(pawn_is_isolated(&board, Square::A2, Color::White));
        assert!(pawn_is_isolated(&board, Square::C2, Color::White));

        let connected = parse_board("4k3/8/8/8/8/8/PP6/4K3 w - - 0 1");
        assert!(!pawn_is_isolated(&connected, Square::A2, Color::White));
        assert!(!pawn_is_isolated(&connected, Square::B2, Color::White));
    }

    #[test]
    fn rook_on_seventh_detects_relative_seventh() {
        assert!(rook_on_seventh(Square::A7, Color::White, Square::E8));
        assert!(!rook_on_seventh(Square::A7, Color::White, Square::E7));
        assert!(rook_on_seventh(Square::A2, Color::Black, Square::E1));
        assert!(!rook_on_seventh(Square::A1, Color::Black, Square::E1));
    }

    #[test]
    fn open_file_rook_scores_higher_than_semi_open_with_same_material() {
        // both sides have one pawn; only the a-file occupancy changes
        let open = parse_board("4k3/1p6/8/8/8/8/1P6/R3K3 w - - 0 1");
        let semi = parse_board("4k3/p7/8/8/8/8/1P6/R3K3 w - - 0 1");
        assert!(has_open_file(&open, Square::A1, Color::White));
        assert!(!has_open_file(&semi, Square::A1, Color::White));
        assert!(has_semi_open_file(&semi, Square::A1, Color::White));
        assert!(pesto_evaluate_from_scratch(&open) > pesto_evaluate_from_scratch(&semi));
    }
}