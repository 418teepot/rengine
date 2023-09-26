use crate::gamestate::{Piece, Side};
use crate::bitboard::Square;

#[derive(Default)]
pub struct ZobristHash(i64);

impl ZobristHash {
    pub fn add_piece(&mut self, square: Square, piece: Piece, side: Side) {
        todo!()
    }

    pub fn remove_piece(&mut self, square: Square, piece: Piece, side: Side) {
        todo!()
    }

    pub fn add_castling_right(&mut self, right: usize) {
        todo!()
    }

    pub fn remove_castling_right(&mut self, right: usize) {
        todo!()
    }

    pub fn flip_side_to_move(&mut self) {
        todo!()
    }

    pub fn init_side_to_move(&mut self, side: Side) {
        todo!()
    }
    
    pub fn add_en_passant_square(&mut self, square: Square) {
        todo!()
    }

    pub fn remove_en_passant_square(&mut self, square: Square) {
        todo!()
    }
}

lazy_static! {
}