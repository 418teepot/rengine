use rand::SeedableRng;

use crate::bitboard::Square;
use crate::gamestate::{Piece, PAWN};
use std::ops::BitOr;

const MAX_MOVES: usize = 100;
const NULLMOVE: Move = Move(0);

/// Represents a chess move. Captured piece and moving piece is stored here as well for fast undo_move().
/// 
/// # ccc ssss mmm tttttt ffffff
/// 
/// * ffffff (0-5): The square the piece is moving from
/// * tttttt (6-11): The square the piece is moving to
/// * mmm (12-15): The piece that is being moved
/// * ssss (16-19): Special move information
/// * Special0 (16):  Double Pawn Push or Ep-Capture or Promoted Piece
/// * Special1 (17):  Castle or Promoted piece
/// * Capture (18): Is the move a capture
/// * Promotion (19): Is the move a promotion
/// * ccc (20-22): The piece that was captured if there was a capture
/// 
/// https://www.chessprogramming.org/Encoding_Moves#From-To_Based
#[derive(Default, Copy, Clone)]
pub struct Move(pub u32);

const MOVE_TO_OFFSET: usize = 6;
const MOVE_MOVING_OFFSET: usize = 12;
const MOVE_CAPTURED_OFFSET: usize = 20;
const MOVE_IS_CAPTURE_OFFSET: usize = 18;
const MOVE_SPECIAL_OFFSET: usize = 16;
const MOVE_PROMOTION_OFFSET: usize = 19;

const FLAG_EN_PASSANT_CAPTURE: u32 = 0b101;
const FLAG_DOUBLE_PAWN_PUSH: u32 = 0b1;
const FLAG_QUIET_PROMOTION: u32 = 0b1000;

const MASK_SQUARE: u32 = 0b111111;
const PROMOTED_PIECE_MASK: u32 = 0b11;
const MASK_PIECE: u32 = 0b111;
const CASTLE_MASK: u32 = 0b11;

const SINGLE_FLAG: u32 = 0b1;


const QUEENSIDE_CASTLE: u32 = 0b11 << MOVE_SPECIAL_OFFSET;
const KINGSIDE_CASTLE: u32 = 0b10 << MOVE_SPECIAL_OFFSET;

pub enum CastlingSide {
    QueenSide,
    KingSide,
}

impl Move {
    fn new_from_to(from: Square, to: Square, moving: Piece) -> Self {
        Move((from as u32) | (to as u32) << MOVE_TO_OFFSET | (moving as u32) << MOVE_MOVING_OFFSET)
    }

    fn new_capture(from: Square, to: Square, moving: Piece, captured: Piece) -> Self {
        Self::new_from_to(from, to, moving) | (captured as u32) << MOVE_CAPTURED_OFFSET | 1 << MOVE_IS_CAPTURE_OFFSET
    }

    fn new_en_passant_capture(from: Square, to: Square) -> Self {
        Self::new_from_to(from, to, PAWN) | FLAG_EN_PASSANT_CAPTURE << MOVE_SPECIAL_OFFSET | (PAWN as u32) << MOVE_MOVING_OFFSET
    }

    fn new_double_pawn_push(from: Square, to: Square) -> Self {
        Self::new_from_to(from, to, PAWN) | FLAG_DOUBLE_PAWN_PUSH << MOVE_SPECIAL_OFFSET
    }

    // TODO: Expand into 2 functions
    fn new_castle(side: CastlingSide) -> Self {
        match side {
            CastlingSide::QueenSide => Move(QUEENSIDE_CASTLE),
            CastlingSide::KingSide => Move(KINGSIDE_CASTLE),
        }
    }

    fn new_quiet_promotion(from: Square, to: Square, piece: Piece) -> Self {
        Self::new_from_to(from, to, PAWN) | FLAG_QUIET_PROMOTION << MOVE_SPECIAL_OFFSET | (piece as u32 - 1) << MOVE_SPECIAL_OFFSET
    }

    fn new_capture_promotion(from: Square, to: Square, piece: Piece, captured: Piece) -> Self {
        Self::new_quiet_promotion(from, to, piece) | (captured as u32) << MOVE_CAPTURED_OFFSET
    }

    pub fn is_capture(&self) -> bool {
        SINGLE_FLAG & (self.0 >> MOVE_IS_CAPTURE_OFFSET) == 1
    }

    pub fn captured_piece(&self) -> Piece {
        (self.0 >> MOVE_CAPTURED_OFFSET) as Piece
    }

    pub fn from(&self) -> Square {
        (MASK_SQUARE & self.0) as Square
    }

    pub fn to(&self) -> Square {
        (MASK_SQUARE & (self.0 >> MOVE_TO_OFFSET)) as Square
    }

    pub fn is_promotion(&self) -> bool {
        (self.0 >> MOVE_PROMOTION_OFFSET) & SINGLE_FLAG == 1
    }

    pub fn is_capture_and_en_passant(&self) -> bool {
        (self.0 >> MOVE_SPECIAL_OFFSET) & FLAG_EN_PASSANT_CAPTURE != 0
    }

    pub fn promoted_piece(&self) -> Piece {
        (((self.0 >> MOVE_SPECIAL_OFFSET) & PROMOTED_PIECE_MASK) + 1) as Piece
    }

    pub fn moving_piece(&self) -> Piece {
        ((self.0 >> MOVE_MOVING_OFFSET) & MASK_PIECE) as Piece
    }

    pub fn is_castle_and_where(&self) -> Option<CastlingSide> {
        let castle_side = self.0 >> MOVE_SPECIAL_OFFSET & CASTLE_MASK;
        if castle_side == 0 {
            None
        }
        else if castle_side == 0b11 {
            Some(CastlingSide::QueenSide)
        }
        else {
            Some(CastlingSide::KingSide)
        }
    }
}

struct MoveList {
    moves: [Move; MAX_MOVES],
    length: u8,
}

impl MoveList {
    fn new() -> MoveList {
        MoveList {
            moves: [NULLMOVE; MAX_MOVES],
            length: 0,
        }
    }

    fn add_move(&mut self, r#move: Move) {
        assert!(self.length as usize <= MAX_MOVES);
        self.moves[self.length as usize] = r#move;
        self.length += 1;
    }
}

struct MoveIterator {
    move_list: MoveList,
    index: u8,
}

impl Iterator for MoveIterator {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.move_list.length - 1 {
            return None;
        }
        let r#move = self.move_list.moves[self.index as usize];
        self.index += 1;
        Some(r#move)
    }
}

impl IntoIterator for MoveList {
    type Item = Move;

    type IntoIter = MoveIterator;

    fn into_iter(self) -> Self::IntoIter {
        MoveIterator {
            move_list: self,
            index: 0,
        }
    }
}

impl BitOr<u32> for Move {
    type Output = Move;

    #[inline(always)]
    fn bitor(self, rhs: u32) -> Self::Output {
        Move(self.0 | rhs)
    }
}