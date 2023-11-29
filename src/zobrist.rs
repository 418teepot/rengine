use crate::gamestate::{Piece, Side, NUM_OF_PIECES, NUM_OF_PLAYERS, BLACK};
use crate::bitboard::{Square, Bitboard, NUM_OF_SQUARES};
use rand::Rng;

#[derive(Default, Copy, Clone, PartialEq, Debug)]
pub struct ZobristHash(pub u64);

impl ZobristHash {
    #[inline(always)]
    pub fn add_piece(&mut self, square: Square, piece: Piece, side: Side) {
        self.0 ^= ZOBRIST_PIECES[side][piece][square].0;
    }

    #[inline(always)]
    pub fn remove_piece(&mut self, square: Square, piece: Piece, side: Side) {
        self.add_piece(square, piece, side);
    }

    #[inline(always)]
    pub fn add_castling_right(&mut self, right: usize) {
        self.0 ^= ZOBRIST_CASTLING_RIGHTS[right].0;
    }

    #[inline(always)]
    pub fn remove_castling_right(&mut self, right: usize) {
        self.add_castling_right(right);
    }

    #[inline(always)]
    pub fn flip_side_to_move(&mut self) {
        self.0 ^= ZOBRIST_BLACK_TO_MOVE.0;
    }

    #[inline(always)]
    pub fn init_side_to_move(&mut self, side: Side) {
        if side == BLACK {
            self.0 ^= ZOBRIST_BLACK_TO_MOVE.0;
        }
    }

    #[inline(always)]
    pub fn add_en_passant_square(&mut self, square: Square) {
        self.0 ^= ZOBRIST_EN_PASSANT_SQUARE[square].0;
    }

    #[inline(always)]
    pub fn remove_en_passant_square(&mut self, square: Square) {
        self.add_en_passant_square(square);
    }

    #[inline(always)]
    pub fn init_en_passant_square(&mut self, board: Bitboard) {
        if !board.is_empty() {
            self.add_en_passant_square(board.next_piece_index());
        }
    }
}

lazy_static! {
    static ref ZOBRIST_PIECES: [[[ZobristHash; NUM_OF_SQUARES]; NUM_OF_PIECES]; NUM_OF_PLAYERS] = {
        let mut rng = rand::thread_rng();
        let mut zobrist_pieces = [[[ZobristHash(0); NUM_OF_SQUARES]; NUM_OF_PIECES]; NUM_OF_PLAYERS];
        for square in 0..NUM_OF_SQUARES {
            for piece in 0..NUM_OF_PIECES {
                for player in 0..NUM_OF_PLAYERS {
                    zobrist_pieces[player][piece][square] = ZobristHash(rng.gen());
                }
            }
        }
        zobrist_pieces
    };

    static ref ZOBRIST_BLACK_TO_MOVE: ZobristHash = {
        ZobristHash(rand::thread_rng().gen())  
    };

    static ref ZOBRIST_CASTLING_RIGHTS: [ZobristHash; 4] = {
        let mut rng = rand::thread_rng();
        let mut zobrist_castling_rights = [ZobristHash(0); 4];
        for right in 0..4 {
            zobrist_castling_rights[right] = ZobristHash(rng.gen());
        }
        zobrist_castling_rights
    };

    static ref ZOBRIST_EN_PASSANT_SQUARE: [ZobristHash; NUM_OF_SQUARES] = {
        let mut rng = rand::thread_rng();
        let mut zobrist_en_passant_square = [ZobristHash(0); NUM_OF_SQUARES];
        for square in 0..NUM_OF_SQUARES {
            zobrist_en_passant_square[square] = ZobristHash(rng.gen());
        }
        zobrist_en_passant_square
    };
}