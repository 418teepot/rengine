use crate::eval::{MATERIAL_VALUE, PHASE_WEIGHT, EVAL_PARAMS};
use crate::movegen::{CASTLE_WHITE_QUEENSIDE_CHECK_FREE, CASTLE_WHITE_KINGSIDE_CHECK_FREE, CASTLE_BLACK_QUEENSIDE_CHECK_FREE, CASTLE_BLACK_KINGSIDE_CHECK_FREE};
use crate::smpsearch::{Eval, NULLMOVE};
use crate::bitboard::{Bitboard, Square};
use crate::r#move::{Move, CastlingSide, self};
use crate::uci::algebraic_to_index;
use crate::zobrist::ZobristHash;

// TODO: Maybe smaller sizes?
pub type Piece = usize;
pub type Side = usize;
type Ply = usize;

pub const NUM_OF_PIECES: usize = 6;
pub const NUM_OF_PLAYERS: usize = 2;

pub const PAWN: Piece = 0;
pub const ROOK: Piece = 1;
pub const KNIGHT: Piece = 2;
pub const BISHOP: Piece = 3;
pub const QUEEN: Piece = 4;
pub const KING: Piece = 5;

pub const WHITE: Side = 0;
pub const BLACK: Side = 1;

pub const WHITE_KINGSIDE_CASTLE: usize = 1;
pub const WHITE_QUEENSIDE_CASTLE: usize = 0;
pub const BLACK_QUEENSIDE_CASTLE: usize = 2;
pub const BLACK_KINGSIDE_CASTLE: usize = 3;

pub const A8: Square = 56;
pub const H8: Square = 63;
pub const D8: Square = 59;
pub const C8: Square = 58;
pub const G8: Square = 62;
pub const F8: Square = 61;
pub const A1: Square = 0;
pub const H1: Square = 7;
pub const F1: Square = 5;
pub const D1: Square = 3;
pub const E1: Square = 4;
pub const G1: Square = 6;
pub const C1: Square = 2;
pub const E8: Square = 60;


#[derive(Default, PartialEq, Clone)]
pub struct GameState {
    pub piece_boards: [[Bitboard; NUM_OF_PIECES]; NUM_OF_PLAYERS],
    pub plys: Ply,
    pub en_passant_board: Bitboard,
    pub castling_rights: [bool; 4],
    pub fifty_move_rule: Ply,
    pub zobrist: ZobristHash,
    pub history: Vec<History>,
    // Eval
    pub material: [Eval; NUM_OF_PLAYERS],
    pub material_eg: [Eval; NUM_OF_PLAYERS],
    pub psqt_mg: [Eval; NUM_OF_PLAYERS],
    pub phase: i16,
    pub psqt_eg: [Eval; NUM_OF_PLAYERS],
    pub has_castled: [bool; NUM_OF_PLAYERS],
    pub search_ply: u8,
}

impl GameState {
    pub fn new_from_fen(fen: &str) -> Self {
        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.len() != 6 {
            panic!("Invalid number of parts in fen string.");
        }

        let mut state: GameState = Default::default();

        let piece_structure = parts[0];
        let _side_to_move = parts[1];
        let castling_rights = parts[2];
        let en_passant_square = parts[3];
        let fifty_move_clock = parts[4];
        let ply_clock = parts[5];

        let mut file = 0;
        let mut rank = 7;

        for character in piece_structure.chars() {
            let square = rank_file_to_square(file, rank);
            match character {
                '/' => {
                    file = 0;
                    rank -= 1;
                    continue;
                },
                '1'..='8' => {
                    file += character.to_digit(10).unwrap() as usize;
                    continue;
                },
                'p' => { state.add_piece(square, PAWN, BLACK); },
                'r' => { state.add_piece(square, ROOK, BLACK); },
                'b' => { state.add_piece(square, BISHOP, BLACK); },
                'n' => { state.add_piece(square, KNIGHT, BLACK); },
                'q' => { state.add_piece(square, QUEEN, BLACK); },
                'k' => { state.add_piece(square, KING, BLACK); },
                'P' => { state.add_piece(square, PAWN, WHITE); },
                'R' => { state.add_piece(square, ROOK, WHITE); },
                'B' => { state.add_piece(square, BISHOP, WHITE); },
                'N' => { state.add_piece(square, KNIGHT, WHITE); },
                'Q' => { state.add_piece(square, QUEEN, WHITE); },
                'K' => { state.add_piece(square, KING, WHITE); },
                _ => panic!("Invalid character in fen string piece list."),
            }
            file += 1;
        }

        // state.plys = ply_clock.parse().expect("Ply clock is not a number in fen string.");
        state.plys = if _side_to_move == "w" {
            0
        } else {
            1
        };
        state.zobrist.init_side_to_move(state.side_to_move());
        state.fifty_move_rule = fifty_move_clock.parse().expect("50 move rule clock is not a valid number in fen string.");
        state.en_passant_board = match en_passant_square {
            "-" => Bitboard::empty(),
            _ => Bitboard::square(algebraic_to_index(en_passant_square).unwrap()),
        };
        state.zobrist.init_en_passant_square(state.en_passant_board);

        for character in castling_rights.chars() {
            match character {
                '-' => (),
                'K' => state.add_castling_right(WHITE_KINGSIDE_CASTLE),
                'Q' => state.add_castling_right(WHITE_QUEENSIDE_CASTLE),
                'k' => state.add_castling_right(BLACK_KINGSIDE_CASTLE),
                'q' => state.add_castling_right(BLACK_QUEENSIDE_CASTLE),
                _ => panic!("Invalid character in castling rights of fen str."),
            }
        }

        state
    }

    pub fn to_reduced_book_fen(&self) -> String {
        let mut fen_string = String::new();

        let mut empty_stack: u32 = 0;
        for rank in (0..8).rev() {
            for file in 0..8 {
                let square = rank * 8 + file;
                if square % 8 == 0 {
                    if empty_stack > 0 {
                        fen_string.push(std::char::from_digit(empty_stack, 10).unwrap());
                    }
                    if !fen_string.is_empty() {
                        fen_string.push('/');
                    }
                    empty_stack = 0;
                }
                match self.find_piece_on_all(square) {
                    None => {
                        empty_stack += 1;
                    },
                    Some((side, piece)) => {
                        if empty_stack > 0 {
                            fen_string.push(std::char::from_digit(empty_stack, 10).unwrap());
                        }
                        let piece_char = piece_to_char(side, piece);
                        fen_string.push(piece_char);
                        empty_stack = 0;
                    }
                }
            }
        }
        if empty_stack > 0 {
            fen_string.push(std::char::from_digit(empty_stack, 10).unwrap());
        }

        fen_string.push(' ');
        fen_string.push(if self.side_to_move() == WHITE { 'w' } else { 'b' });
        fen_string.push(' ');

        let mut some_castle = false;
        if self.castling_rights[WHITE_KINGSIDE_CASTLE] {
            fen_string.push('K');
            some_castle = true;
        }
        if self.castling_rights[WHITE_QUEENSIDE_CASTLE] {
            fen_string.push('Q');
            some_castle = true;
        }
        if self.castling_rights[BLACK_KINGSIDE_CASTLE] {
            fen_string.push('k');
            some_castle = true;
        }
        if self.castling_rights[BLACK_QUEENSIDE_CASTLE] {
            fen_string.push('q');
            some_castle = true;
        }

        if !some_castle {
            fen_string.push('-');
        }

        fen_string.push(' ');
        fen_string.push('-');

        fen_string
    }

    pub fn new_starting_pos() -> Self {
        Self::new_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }

    pub fn apply_legal_move(&mut self, r#move: Move) {
        assert!(r#move != NULLMOVE);
        self.history.push(History { r#move, en_passant: self.en_passant_board, fifty_move_rule: self.fifty_move_rule, castling_rights: self.castling_rights, zobrist: self.zobrist });
        let from = r#move.from();
        let to = r#move.to();
        let our_side = self.side_to_move();
        let enemy_side = our_side ^ 1;
        let moving_piece = r#move.moving_piece();

        if self.en_passant_board.is_filled() {
            self.zobrist.remove_en_passant_square(self.en_passant_board.next_piece_index());
            self.en_passant_board = Bitboard::empty();
        }
        
        self.fifty_move_rule += 1;
        self.plys += 1;
        self.search_ply += 1;
        self.zobrist.flip_side_to_move();

        if r#move.is_capture() {
            self.fifty_move_rule = 0;

            if r#move.is_capture_and_en_passant() {
                self.remove_en_passant_pawn(to, our_side);
            }
            else {
                let captured = r#move.captured_piece();
                self.remove_piece(to, captured, enemy_side);
                if captured == ROOK {
                    self.handle_rook_capture(to, our_side);
                }
            }
        }
        
        if r#move.is_promotion() {
            self.fifty_move_rule = 0;
            let promoted = r#move.promoted_piece();
            self.add_piece(to, promoted, our_side);
            self.remove_piece(from, moving_piece, our_side);
            return
        }
        
        if let Some(castle_side) = r#move.is_castle_and_where() {
            self.do_castle(castle_side, our_side);
            return
        }

        if r#move.is_double_pawn_push() {
            self.handle_double_pawn_push(to, our_side);
        }

        self.handle_castling_right_removing_moves(from, moving_piece, our_side);

        self.move_piece(from, to, moving_piece, our_side);
        
    }

    pub fn apply_pseudo_legal_move(&mut self, r#move: Move) -> bool {
        let our_side = self.side_to_move();
        let enemy_side = our_side ^ 1;

        if let Some(side) = r#move.is_castle_and_where() {
            let blockers = self.occupancy(WHITE) | self.occupancy(BLACK);
            if our_side == WHITE {
                if side == r#move::CastlingSide::QueenSide {
                    for to_sq in CASTLE_WHITE_QUEENSIDE_CHECK_FREE {
                        if self.attackers_on_square(to_sq, enemy_side, blockers).is_filled() {
                            return false;
                        }
                    }
                    self.apply_legal_move(r#move);
                    return true;
                }
                else if side == r#move::CastlingSide::KingSide {
                    for to_sq in CASTLE_WHITE_KINGSIDE_CHECK_FREE {
                        if self.attackers_on_square(to_sq, enemy_side, blockers).is_filled() {
                            return false;
                        } 
                    }
                    self.apply_legal_move(r#move);
                    return true;
                }
            } 
            else if side == r#move::CastlingSide::QueenSide {
                    for to_sq in CASTLE_BLACK_QUEENSIDE_CHECK_FREE {
                        if self.attackers_on_square(to_sq, enemy_side, blockers).is_filled() {
                            return false;
                        }
                    }
                    self.apply_legal_move(r#move);
                    return true;
            }
            else if side == r#move::CastlingSide::KingSide {
                for to_sq in CASTLE_BLACK_KINGSIDE_CHECK_FREE {
                    if self.attackers_on_square(to_sq, enemy_side, blockers).is_filled() {
                        return false
                    }
                }
                self.apply_legal_move(r#move);
                return true;
            }
        }

        self.apply_legal_move(r#move);
        if self.attackers_on_square(self.piece_boards[our_side][KING].next_piece_index(), enemy_side, self.occupancy(WHITE) | self.occupancy(BLACK)).is_filled() {
            self.undo_move();
            return false;
        }
        
        true

    }

    pub fn find_first_legal_move(&mut self) -> Move {
        for r#move in self.generate_pseudo_legal_moves() {
            if self.apply_pseudo_legal_move(r#move) {
                self.undo_move();
                return r#move;
            }
        }
        unreachable!()
    }

    pub fn make_null_move(&mut self) {
        self.history.push(History { r#move: Move::new_from_to(0, 0, 0), fifty_move_rule: self.fifty_move_rule, castling_rights: self.castling_rights, zobrist: self.zobrist, en_passant: self.en_passant_board });
        self.plys += 1;
        if self.en_passant_board != Bitboard(0) {
            self.zobrist.remove_en_passant_square(self.en_passant_board.next_piece_index());
            self.en_passant_board = Bitboard(0);
        }
        self.zobrist.flip_side_to_move();
    }

    pub fn undo_null_move(&mut self) {
        self.plys -= 1;
        let history = self.history.pop().unwrap();
        self.zobrist = history.zobrist;
        self.fifty_move_rule = history.fifty_move_rule;
        self.en_passant_board = history.en_passant;
        self.castling_rights = history.castling_rights;
    }

    fn handle_castling_right_removing_moves(&mut self, from: Square, moving_piece: Piece, our_side: Side) {
        if moving_piece == PAWN {
            self.fifty_move_rule = 0;
        }
        else if moving_piece == KING {
            self.handle_king_move(our_side);
        }
        else if moving_piece == ROOK {
            self.handle_rook_move(from, our_side);
        }
    }

    fn handle_king_move(&mut self, our_side: Side) {
        if our_side == WHITE {
            self.remove_castling_right(WHITE_KINGSIDE_CASTLE);
            self.remove_castling_right(WHITE_QUEENSIDE_CASTLE);
        }
        else {
            self.remove_castling_right(BLACK_KINGSIDE_CASTLE);
            self.remove_castling_right(BLACK_QUEENSIDE_CASTLE);
        }
    }

    fn handle_rook_move(&mut self, from: Square, our_side: Side) {
        if our_side == WHITE {
            if from == A1 {
                self.remove_castling_right(WHITE_QUEENSIDE_CASTLE);
            }
            else if from == H1 {
                self.remove_castling_right(WHITE_KINGSIDE_CASTLE);
            }
        }
        else if from == A8 {
            self.remove_castling_right(BLACK_QUEENSIDE_CASTLE);
        }
        else if from == H8 {
            self.remove_castling_right(BLACK_KINGSIDE_CASTLE);
        }
        
    }

    pub fn undo_move(&mut self) {
        let past = self.history.pop().expect("Tried to undo move that does not exist.");
        self.plys -= 1;
        self.search_ply -= 1;

        let r#move = past.r#move;
        assert!(r#move != Move::new_from_to(0, 0, 0));
        let to = r#move.to();
        let from = r#move.from();
        let our_side = self.side_to_move();
        let enemy_side = our_side ^ 1;
        let moving_piece = r#move.moving_piece();
        
        if r#move.is_capture() {
            if r#move.is_capture_and_en_passant() {
                self.re_add_en_passant_pawn(to, our_side);
            }
            else {
                self.add_piece(to, r#move.captured_piece(), enemy_side);
            }
        }

        if r#move.is_promotion() {
            let promoted = r#move.promoted_piece();
            self.add_piece(from, moving_piece, our_side);
            self.remove_piece(to, promoted, our_side);
            return;
        }

        if let Some(castle_side) = r#move.is_castle_and_where() {
            self.undo_castle(castle_side, our_side);
        } else {
            self.move_piece(to, from, moving_piece, our_side);
        }

        self.castling_rights = past.castling_rights;
        self.fifty_move_rule = past.fifty_move_rule;
        self.en_passant_board = past.en_passant;
        self.zobrist = past.zobrist;
    }

    fn handle_double_pawn_push(&mut self, to: Square, our_side: Side) {
        if our_side == WHITE {
            self.en_passant_board = Bitboard::square(to - 8);
            self.zobrist.add_en_passant_square(to - 8);
        }
        else {
            self.en_passant_board = Bitboard::square(to + 8);
            self.zobrist.add_en_passant_square(to + 9)
        }
    }

    fn do_castle(&mut self, casling_side: CastlingSide, our_side: Side) {
        self.has_castled[our_side] = true;
        if our_side == WHITE {
            match casling_side {
                CastlingSide::QueenSide => {
                    self.move_piece(A1, D1, ROOK, our_side);
                    self.move_piece(E1, C1, KING, our_side);
                },
                CastlingSide::KingSide => {
                    self.move_piece(H1, F1, ROOK, our_side);
                    self.move_piece(E1, G1, KING, our_side);
                },
            }
            self.remove_castling_right(WHITE_KINGSIDE_CASTLE);
            self.remove_castling_right(WHITE_QUEENSIDE_CASTLE);
            return
        }
        match casling_side {
            CastlingSide::QueenSide => {
                self.move_piece(A8, D8, ROOK, our_side);
                self.move_piece(E8, C8, KING, our_side);
            },
            CastlingSide::KingSide => {
                self.move_piece(H8, F8, ROOK, our_side);
                self.move_piece(E8, G8, KING, our_side);
            },
        }
        self.remove_castling_right(BLACK_KINGSIDE_CASTLE);
        self.remove_castling_right(BLACK_QUEENSIDE_CASTLE);

    }
    
    fn undo_castle(&mut self, castling_side: CastlingSide, our_side: Side) {
        self.has_castled[our_side] = false;
        if our_side == WHITE {
            match castling_side {
                CastlingSide::QueenSide => {
                    self.move_piece(D1, A1, ROOK, our_side);
                    self.move_piece(C1, E1, KING, our_side);
                    self.zobrist.add_castling_right(WHITE_QUEENSIDE_CASTLE);
                },
                CastlingSide::KingSide => {
                    self.move_piece(F1, H1, ROOK, our_side);
                    self.move_piece(G1, E1, KING, our_side);
                    self.zobrist.add_castling_right(WHITE_KINGSIDE_CASTLE);
                }
            }
            return;
        }
        match castling_side {
            CastlingSide::QueenSide => {
                self.move_piece(D8, A8, ROOK, our_side);
                self.move_piece(C8, E8, KING, our_side);
                self.zobrist.add_castling_right(BLACK_QUEENSIDE_CASTLE);
            },
            CastlingSide::KingSide => {
                self.move_piece(F8, H8, ROOK, our_side);
                self.move_piece(G8, E8, KING, our_side);
                self.zobrist.add_castling_right(BLACK_KINGSIDE_CASTLE);
            }
        }
    }

    fn remove_en_passant_pawn(&mut self, to: Square, our_side: Side) {
        if our_side == WHITE {
            self.remove_piece(to - 8, PAWN, our_side ^ 1);
        }
        else {
            self.remove_piece(to + 8, PAWN, our_side ^ 1);
        }
    }
    
    fn re_add_en_passant_pawn(&mut self, to: Square, our_side: Side) {
        if our_side == WHITE {
            self.add_piece(to - 8, PAWN, our_side ^ 1);
        }
        else {
            self.add_piece(to + 8, PAWN, our_side ^ 1);
        }
    }


    fn handle_rook_capture(&mut self, to: Square, our_side: Side) {
        if our_side == WHITE {
            if to == A8 {
                self.remove_castling_right(BLACK_QUEENSIDE_CASTLE);
            }
            else if to == H8 {
                self.remove_castling_right(BLACK_KINGSIDE_CASTLE);
            }
        }
        else if to == A1 {
                self.remove_castling_right(WHITE_QUEENSIDE_CASTLE);
            }
            else if to == H1 {
                self.remove_castling_right(WHITE_KINGSIDE_CASTLE);
            }
        
    }

    pub fn is_game_over(&mut self) -> bool {
        let moves = self.generate_pseudo_legal_moves();
        if self.piece_boards[WHITE][PAWN].is_empty() && self.piece_boards[BLACK][PAWN].is_empty() && self.is_material_draw() {
            return true;
        }
        if self.fifty_move_rule >= 100 || self.has_repitition() {
            return true;
        }
        for r#move in moves {
            if self.apply_pseudo_legal_move(r#move) {
                self.undo_move();
                return false;
            }
        }
        true
    }

    pub fn unavoidable_game_over(&mut self) -> bool {
        for r#move in self.generate_pseudo_legal_moves() {
            if self.apply_pseudo_legal_move(r#move) {
                self.undo_move();
                return false;
            }
        }
        true
    }

    pub fn dump_panic_debug(&self) {
        println!("\nFen_str: {}\n", self.to_reduced_book_fen());
        print!("Move history: ");
        for his in self.history.clone() {
            print!("{} ", his.r#move.to_algebraic());
        }
        println!();
        let pseudo_legal_moves = self.generate_pseudo_legal_moves();
        print!("Pseudo legal moves: ");
        for r#move in pseudo_legal_moves {
            print!("{} ", r#move.to_algebraic());
        }
        println!();
        
    }

    #[inline(always)]
    fn add_piece(&mut self, square: Square, piece: Piece, side: Side) {
        self.piece_boards[side][piece].add_piece(square);
        self.zobrist.add_piece(square, piece, side);
        unsafe {
            self.psqt_mg[side] += EVAL_PARAMS.psqt_mg[side][piece][square];
            self.psqt_eg[side] += EVAL_PARAMS.psqt_eg[side][piece][square];
            self.material[side] += EVAL_PARAMS.mg_piece_value[piece];
            self.material_eg[side] += EVAL_PARAMS.eg_piece_value[piece];
        }
        self.phase -= PHASE_WEIGHT[piece];
    }

    #[inline(always)]
    fn remove_piece(&mut self, square: Square, piece: Piece, side: Side) {
        self.piece_boards[side][piece].remove_piece(square);
        self.zobrist.remove_piece(square, piece, side);
        unsafe {
            self.psqt_mg[side] -= EVAL_PARAMS.psqt_mg[side][piece][square];
            self.psqt_eg[side] -= EVAL_PARAMS.psqt_eg[side][piece][square];
            self.material[side] -= EVAL_PARAMS.mg_piece_value[piece];
            self.material_eg[side] -= EVAL_PARAMS.eg_piece_value[piece];
        }
        self.phase += PHASE_WEIGHT[piece];
    }

    #[inline(always)]
    fn move_piece(&mut self, from: Square, to: Square, piece: Piece, side: Side) {
        self.remove_piece(from, piece, side);
        self.add_piece(to, piece, side);
    }

    #[inline(always)]
    fn add_castling_right(&mut self, right: usize) {
        if !self.castling_rights[right] {
            self.castling_rights[right] = true;
            self.zobrist.add_castling_right(right);
        }
    }
    
    fn remove_castling_right(&mut self, right: usize) {
        if self.castling_rights[right] {
            self.castling_rights[right] = false;
            self.zobrist.remove_castling_right(right);
        }
    }

    #[inline(always)]
    pub fn side_to_move(&self) -> Side {
        self.plys & 1
    }

    #[allow(dead_code)]
    pub fn print_debug(&self) {
        let mut rank: isize = 7;
        let mut file: isize = 0;
        loop {
            let square = rank * 8 + file;
            print!("{}",
                match self.find_piece_on_all(square as usize) {
                    None => '.',
                    Some((side, piece)) => {
                        match side {
                            WHITE => {
                                match piece {
                                    PAWN => 'P',
                                    ROOK => 'R',
                                    BISHOP => 'B',
                                    KNIGHT => 'N',
                                    QUEEN => 'Q',
                                    KING => 'K',
                                    _ => unreachable!(),
                                }
                            },
                            BLACK => {
                                match piece {
                                PAWN => 'p',
                                ROOK => 'r',
                                BISHOP => 'b',
                                KNIGHT => 'n',
                                QUEEN => 'q',
                                KING => 'k',
                                _ => unreachable!(),
                                }
                            },
                            _ => unreachable!(),
                        }
                    },
                }
            );

            file += 1;
            if file == 8 {
                println!();
                file = 0;
                rank -= 1;
                if rank < 0 {
                    return
                }
            }
        }
    }
}

#[inline(always)]
fn rank_file_to_square(file: Square, rank: Square) -> Square {
    rank * 8 + file
}

fn piece_to_char(side: Side, piece: Piece) -> char {
    let piece = match piece {
        PAWN => 'p',
        ROOK => 'r',
        BISHOP => 'b',
        KNIGHT => 'n',
        QUEEN => 'q',
        KING => 'k',
        _ => unreachable!(),
    };
    if side == WHITE {
        piece.to_ascii_uppercase()
    } else {
        piece
    }
}

#[derive(Default, PartialEq, Clone, Copy)]
pub struct History {
    pub r#move: Move,
    pub fifty_move_rule: Ply,
    pub castling_rights: [bool; 4],
    pub zobrist: ZobristHash,
    pub en_passant: Bitboard,
}

#[cfg(test)]
mod tests {

    use super::GameState;

    #[test]
    fn test_do_undo_move() {
        let mut starting_pos = GameState::new_starting_pos();
        let moves = starting_pos.generate_legal_moves();
        for m in moves {
            let old_state = starting_pos.clone();
            starting_pos.apply_legal_move(m);
            starting_pos.undo_move();
            if old_state == starting_pos {
            } else {
                assert_eq!(true, false);
            }
        }
    }
}