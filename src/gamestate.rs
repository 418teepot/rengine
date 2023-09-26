use crate::bitboard::{Bitboard, Square};
use crate::r#move::Move;
use crate::zobrist::ZobristHash;

// TODO: Maybe smaller sizes?
pub type Piece = usize;
pub type Side = usize;
type Ply = usize;

const NUM_OF_PIECES: usize = 6;
const NUM_OF_PLAYERS: usize = 2;

const PAWN: Piece = 0;
const ROOK: Piece = 1;
const KNIGHT: Piece = 2;
const BISHOP: Piece = 3;
const QUEEN: Piece = 4;
const KING: Piece = 5;

const WHITE: Side = 0;
const BLACK: Side = 1;

const WHITE_KINGSIDE_CASTLE: usize = 1;
const WHITE_QUEENSIDE_CASTLE: usize = 0;
const BLACK_QUEENSIDE_CASTLE: usize = 2;
const BLACK_KINGSIDE_CASTLE: usize = 3;

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

    #[inline(always)]
    fn add_piece(&mut self, square: Square, piece: Piece, side: Side) {
        self.piece_boards[side][piece].add_piece(square);
        self.zobrist.add_piece(square, piece, side);
    }

    #[inline(always)]
    fn remove_piece(&mut self, square: Square, piece: Piece, side: Side) {
        self.piece_boards[side][piece].remove_piece(square);
        self.zobrist.remove_piece(square, piece, side);
    }

    #[inline(always)]
    fn add_castling_right(&mut self, right: usize) {
        self.castling_rights[right] = true;
        self.zobrist.add_castling_right(right);
    }
    
    #[inline(always)]
    fn remove_castling_right(&mut self, right: usize) {
        self.castling_rights[right] = false;
        self.zobrist.remove_castling_right(right);
    }

    #[inline(always)]
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