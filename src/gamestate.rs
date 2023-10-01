use crate::bitboard::{Bitboard, Square};
use crate::r#move::{Move, CastlingSide};
use crate::zobrist::ZobristHash;

// TODO: Maybe smaller sizes?
pub type Piece = usize;
pub type Side = usize;
type Ply = usize;

pub const NUM_OF_PIECES: usize = 6;
pub const NUM_OF_PLAYERS: usize = 2;

pub const PAWN: Piece = 0;
pub const ROOK: Piece = 1;
const KNIGHT: Piece = 2;
pub const BISHOP: Piece = 3;
const QUEEN: Piece = 4;
const KING: Piece = 5;

pub const WHITE: Side = 0;
pub const BLACK: Side = 1;

const WHITE_KINGSIDE_CASTLE: usize = 1;
const WHITE_QUEENSIDE_CASTLE: usize = 0;
const BLACK_QUEENSIDE_CASTLE: usize = 2;
const BLACK_KINGSIDE_CASTLE: usize = 3;

const A8: Square = 56;
const H8: Square = 63;
const D8: Square = 59;
const C8: Square = 58;
const G8: Square = 62;
const F8: Square = 61;
const A1: Square = 0;
const H1: Square = 7;
const F1: Square = 5;
const D1: Square = 3;
const E1: Square = 4;
const G1: Square = 6;
const C1: Square = 2;
const E8: Square = 60;


#[derive(Default)]
struct GameState {
    piece_boards: [[Bitboard; NUM_OF_PIECES]; NUM_OF_PLAYERS],
    plys: Ply,
    en_passant_board: Bitboard,
    castling_rights: [bool; 4],
    fifty_move_rule: Ply,
    zobrist: ZobristHash,
    history: Vec<History>,
}

impl GameState {
    // TODO: Recoverable error for malformed fen string
    fn new_from_fen(fen: &str) -> Self {
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
        let mut rank = 0;

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

        state.plys = ply_clock.parse().expect("Ply clock is not a number in fen string.");
        state.zobrist.init_side_to_move(state.side_to_move());
        state.fifty_move_rule = fifty_move_clock.parse().expect("50 move rule clock is not a valid number in fen string.");
        state.en_passant_board = match en_passant_square {
            "-" => Bitboard::empty(),
            _ => Bitboard::square(en_passant_square.parse().expect("Not a valid en_passant square in fen string")),
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

    fn new_starting_pos() -> Self {
        Self::new_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }

    fn apply_move(&mut self, r#move: Move) {
        let from = r#move.from();
        let to = r#move.to();
        let our_side = self.side_to_move();
        let enemy_side = our_side ^ 1;
        let moving_piece = r#move.moving_piece();

        self.en_passant_board = Bitboard::empty();
        self.fifty_move_rule += 1;

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

    fn handle_castling_right_removing_moves(&mut self, from: Square, moving_piece: Piece, our_side: Side) {
        if moving_piece == PAWN {
            self.fifty_move_rule = 0;
        }
        else if moving_piece == KING {
            self.handle_king_move(from, our_side);
        }
        else if moving_piece == ROOK {
            self.handle_rook_move(from, our_side);
        }
    }

    fn handle_king_move(&mut self, from: Square, our_side: Side) {
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
        else {
            if from == A8 {
                self.remove_castling_right(BLACK_QUEENSIDE_CASTLE);
            }
            else if from == H8 {
                self.remove_castling_right(BLACK_KINGSIDE_CASTLE);
            }
        }
    }

    fn undo_move(&mut self) {
        let past = self.history.pop().expect("Tried to undo move that does not exist.");
        self.plys -= 1;

        let r#move = past.r#move;
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
        }

        self.move_piece(to, from, moving_piece, our_side);

        self.castling_rights = past.castling_rights;
        self.fifty_move_rule = past.fifty_move_rule;
        self.zobrist = past.zobrist;
    }

    fn handle_double_pawn_push(&mut self, to: Square, our_side: Side) {
        if our_side == WHITE {
            self.en_passant_board = Bitboard::square(to - 8);
        }
        else {
            self.en_passant_board = Bitboard::square(to + 8);
        }
    }

    fn do_castle(&mut self, casling_side: CastlingSide, our_side: Side) {
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
        if our_side == WHITE {
            match castling_side {
                CastlingSide::QueenSide => {
                    self.move_piece(D8, A8, ROOK, our_side);
                    self.move_piece(C8, E8, KING, our_side);
                },
                CastlingSide::KingSide => {
                    self.move_piece(F8, H8, ROOK, our_side);
                    self.move_piece(G8, E8, KING, our_side);
                }
            }
            return;
        }
        match castling_side {
            CastlingSide::QueenSide => {
                self.move_piece(D8, A8, ROOK, our_side);
                self.move_piece(C8, E8, KING, our_side);
            },
            CastlingSide::KingSide => {
                self.move_piece(F8, H8, ROOK, our_side);
                self.move_piece(G8, E8, KING, our_side);
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
        else {
            if to == A1 {
                self.remove_castling_right(WHITE_QUEENSIDE_CASTLE);
            }
            else if to == H1 {
                self.remove_castling_right(WHITE_KINGSIDE_CASTLE);
            }
        }
    }

    fn add_piece(&mut self, square: Square, piece: Piece, side: Side) {
        self.piece_boards[side][piece].add_piece(square);
        self.zobrist.add_piece(square, piece, side);
    }

    fn remove_piece(&mut self, square: Square, piece: Piece, side: Side) {
        self.piece_boards[side][piece].remove_piece(square);
        self.zobrist.remove_piece(square, piece, side);
    }

    fn move_piece(&mut self, from: Square, to: Square, piece: Piece, side: Side) {
        self.remove_piece(from, piece, side);
        self.add_piece(to, piece, side);
    }

    fn add_castling_right(&mut self, right: usize) {
        self.castling_rights[right] = true;
        self.zobrist.add_castling_right(right);
    }
    
    fn remove_castling_right(&mut self, right: usize) {
        if self.castling_rights[right] == true {
            self.castling_rights[right] = false;
            self.zobrist.remove_castling_right(right);
        }
    }

    fn side_to_move(&self) -> Side {
        self.plys & 1
    }
}

fn rank_file_to_square(file: Square, rank: Square) -> Square {
    rank * 8 + file
}

#[derive(Default)]
struct History {
    r#move: Move,
    fifty_move_rule: Ply,
    castling_rights: [bool; 4],
    zobrist: ZobristHash,
}