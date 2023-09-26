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
}

lazy_static! {
}