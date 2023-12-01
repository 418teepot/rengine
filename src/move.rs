use crate::bitboard::Square;
use crate::gamestate::{Piece, PAWN, KING, NUM_OF_PIECES, GameState, WHITE, QUEEN, ROOK, KNIGHT, BISHOP, G1, E1, C1, E8, G8, C8};
use crate::search::Eval;
use crate::uci::algebraic_to_index;
use std::ops::BitOr;

pub const MAX_MOVES: usize = 200;
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
#[derive(Default, Copy, Clone, Debug, PartialEq)]
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
const SPECIAL_MASK: u32 = 0b1111;

const SINGLE_FLAG: u32 = 0b1;


const QUEENSIDE_CASTLE: u32 = 0b11 << MOVE_SPECIAL_OFFSET;
const KINGSIDE_CASTLE: u32 = 0b10 << MOVE_SPECIAL_OFFSET;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum CastlingSide {
    QueenSide,
    KingSide,
}

static PIECE_CHAR: [char; NUM_OF_PIECES] = ['p', 'r', 'n', 'b', 'q', 'k'];

impl Move {
    #[inline(always)]
    pub fn new_from_to(from: Square, to: Square, moving: Piece) -> Self {
        Move((from as u32) | (to as u32) << MOVE_TO_OFFSET | (moving as u32) << MOVE_MOVING_OFFSET)
    }

    #[inline(always)]
    pub fn new_capture(from: Square, to: Square, moving: Piece, captured: Piece) -> Self {
        Self::new_from_to(from, to, moving) | (captured as u32) << MOVE_CAPTURED_OFFSET | 1 << MOVE_IS_CAPTURE_OFFSET
    }

    #[inline(always)]
    pub fn new_en_passant_capture(from: Square, to: Square) -> Self {
        Self::new_from_to(from, to, PAWN) | FLAG_EN_PASSANT_CAPTURE << MOVE_SPECIAL_OFFSET | (PAWN as u32) << MOVE_MOVING_OFFSET
    }

    #[inline(always)]
    pub fn new_double_pawn_push(from: Square, to: Square) -> Self {
        Self::new_from_to(from, to, PAWN) | FLAG_DOUBLE_PAWN_PUSH << MOVE_SPECIAL_OFFSET
    }

    #[inline(always)]
    // TODO: Expand into 2 functions
    pub fn new_castle(side: CastlingSide, from: Square, to: Square) -> Self {
        match side {
            CastlingSide::QueenSide => Self::new_from_to(from, to, KING) | QUEENSIDE_CASTLE,
            CastlingSide::KingSide => Self::new_from_to(from, to, KING) | KINGSIDE_CASTLE,
        }
    }

    #[inline(always)]
    pub fn new_quiet_promotion(from: Square, to: Square, piece: Piece) -> Self {
        Self::new_from_to(from, to, PAWN) | FLAG_QUIET_PROMOTION << MOVE_SPECIAL_OFFSET | (piece as u32 - 1) << MOVE_SPECIAL_OFFSET
    }

    #[inline(always)]
    pub fn new_capture_promotion(from: Square, to: Square, piece: Piece, captured: Piece) -> Self {
        Self::new_quiet_promotion(from, to, piece) | (captured as u32) << MOVE_CAPTURED_OFFSET | 1 << MOVE_IS_CAPTURE_OFFSET
    }

    #[inline(always)]
    pub fn is_capture(&self) -> bool {
        SINGLE_FLAG & (self.0 >> MOVE_IS_CAPTURE_OFFSET) == 1
    }

    #[inline(always)]
    pub fn captured_piece(&self) -> Piece {
        ((self.0 >> MOVE_CAPTURED_OFFSET) & MASK_PIECE) as Piece
    }

    #[inline(always)]
    pub fn from(&self) -> Square {
        (MASK_SQUARE & self.0) as Square
    }

    #[inline(always)]
    pub fn to(&self) -> Square {
        (MASK_SQUARE & (self.0 >> MOVE_TO_OFFSET)) as Square
    }

    #[inline(always)]
    pub fn is_promotion(&self) -> bool {
        (self.0 >> MOVE_PROMOTION_OFFSET) & SINGLE_FLAG == 1
    }

    #[inline(always)]
    pub fn is_capture_and_en_passant(&self) -> bool {
        (self.0 >> MOVE_SPECIAL_OFFSET) & SPECIAL_MASK == FLAG_EN_PASSANT_CAPTURE
    }

    #[inline(always)]
    pub fn promoted_piece(&self) -> Piece {
        (((self.0 >> MOVE_SPECIAL_OFFSET) & PROMOTED_PIECE_MASK) + 1) as Piece
    }

    #[inline(always)]
    pub fn moving_piece(&self) -> Piece {
        ((self.0 >> MOVE_MOVING_OFFSET) & MASK_PIECE) as Piece
    }

    #[inline(always)]
    pub fn is_castle_and_where(&self) -> Option<CastlingSide> {
        let castle_side = (self.0 >> MOVE_SPECIAL_OFFSET) & SPECIAL_MASK;
        if castle_side == 0b11 {
            Some(CastlingSide::QueenSide)
        }
        else if castle_side == 0b10 {
            Some(CastlingSide::KingSide)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn is_double_pawn_push(&self) -> bool {
        ((self.0 >> MOVE_SPECIAL_OFFSET) & SPECIAL_MASK) == 0b1
    }

    #[inline(always)]
    pub fn square_to_algebraic(square: Square) -> String {
        let file = (square % 8) as u8 + b'a';
        let rank = (square / 8) as u8 + b'1';
        format!("{}{}", file as char, rank as char)
    }

    #[inline(always)]
    pub fn to_algebraic(self) -> String {
        if self.is_promotion() {
            format!("{}{}{}", Self::square_to_algebraic(self.from()), Self::square_to_algebraic(self.to()), PIECE_CHAR[self.promoted_piece()]) 
        } else {
            format!("{}{}", Self::square_to_algebraic(self.from()), Self::square_to_algebraic(self.to()))
        }
    }

    pub fn from_text_move(gamestate: &GameState, r#move: &str) -> Self {
        let from = algebraic_to_index(&r#move[0..2]).unwrap();
        let piece_from = gamestate.find_piece_on_all(from).unwrap().1;
        let to = algebraic_to_index(&r#move[2..4]).unwrap();
        let piece_to = gamestate.find_piece_on_all(to);
        if r#move.len() == 5 {
            let promoted_to_char = r#move.chars().nth(4).unwrap();
            let promoted_piece = match promoted_to_char {
                'q' => QUEEN,
                'r' => ROOK,
                'n' => KNIGHT,
                'b' => BISHOP,
                _ => unreachable!(),
            };
            if let Some((_, captured_piece)) = piece_to {
                return Move::new_capture_promotion(from, to, promoted_piece, captured_piece)
            } else {
                return Move::new_quiet_promotion(from, to, piece_from)
            }
        }
        if piece_from == PAWN && to == gamestate.en_passant_board.next_piece_index() {
            return Move::new_en_passant_capture(from, to)
        }
        if piece_from == KING {
            if gamestate.side_to_move() == WHITE {
                if from == E1 {
                    if to == G1 {
                        return Move::new_castle(crate::r#move::CastlingSide::KingSide, from, to)
                        
                    } else if to == C1 {
                        return Move::new_castle(crate::r#move::CastlingSide::QueenSide, from, to)
                    }
                } 
            } else if from == E8 {
                if to == G8 {
                    return Move::new_castle(crate::r#move::CastlingSide::KingSide, from, to)
                } else if to == C8 {
                    return Move::new_castle(crate::r#move::CastlingSide::QueenSide, from, to)
                }
            }    
        }
        if let Some((_, captured_piece)) = piece_to {
            Move::new_capture(from, to, piece_from, captured_piece)
        } else {
            Move::new_from_to(from, to, piece_from)
        }
    }
}

#[derive(Copy, Debug, Clone)]
pub struct MoveList {
    pub moves: [Move; MAX_MOVES],
    pub values: [u32; MAX_MOVES],
    pub length: u8,
}

impl MoveList {
    pub fn new() -> MoveList {
        MoveList {
            moves: [NULLMOVE; MAX_MOVES],
            values: [0; MAX_MOVES],
            length: 0,
        }
    }

    #[inline(always)]
    pub fn add_move(&mut self, r#move: Move) {
        assert!(self.length as usize <= MAX_MOVES);
        self.moves[self.length as usize] = r#move;
        self.length += 1;
    }

}

pub struct MoveIterator {
    move_list: MoveList,
    index: u8,
}

impl Iterator for MoveIterator {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.move_list.length {
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

#[cfg(test)]
mod tests {

    use crate::gamestate::{PAWN, QUEEN, ROOK, KNIGHT};
    use crate::r#move::Move;

    #[test]
    fn test_new_from_to() {
        let from = 63;
        let to = 0;
        let moving = PAWN;
        let r#move = Move::new_from_to(from, to, moving);
        assert_eq!(r#move.from(), from);
        assert_eq!(r#move.to(), to);
        assert_eq!(r#move.moving_piece(), moving);
        assert!(!r#move.is_capture());
        assert!(!r#move.is_capture_and_en_passant());
        assert!(!r#move.is_promotion());
        assert!(!r#move.is_double_pawn_push());
        assert_eq!(r#move.is_castle_and_where(), None);
    }

    #[test]
    fn test_new_capture() {
        let from = 0;
        let to = 63;
        let moving = QUEEN;
        let captured = ROOK;
        let r#move = Move::new_capture(from, to, moving, captured);
        assert_eq!(r#move.from(), from);
        assert_eq!(r#move.to(), to);
        assert_eq!(r#move.moving_piece(), moving);
        assert_eq!(r#move.captured_piece(), captured);
        assert!(r#move.is_capture());
        assert!(!r#move.is_capture_and_en_passant());
        assert!(!r#move.is_double_pawn_push());
        assert!(!r#move.is_promotion());
        assert_eq!(r#move.is_castle_and_where(), None);
    }

    #[test]
    fn test_new_en_passant_capture() {
        let from = 63;
        let to = 31;
        let r#move = Move::new_en_passant_capture(from, to);
        assert_eq!(r#move.from(), from);
        assert_eq!(r#move.to(), to);
        assert_eq!(r#move.moving_piece(), PAWN);
        assert_eq!(r#move.captured_piece(), PAWN);
        assert!(r#move.is_capture());
        assert!(r#move.is_capture_and_en_passant());
        assert!(!r#move.is_double_pawn_push());
        assert!(!r#move.is_promotion());
        assert_eq!(r#move.is_castle_and_where(), None);
    }

    #[test]
    fn test_new_double_pawn_push() {
        let from = 63; 
        let to = 63;
        let r#move = Move::new_double_pawn_push(from, to);
        assert_eq!(r#move.from(), from);
        assert_eq!(r#move.to(), to);
        assert_eq!(r#move.moving_piece(), PAWN);
        assert!(!r#move.is_capture());
        assert!(!r#move.is_capture_and_en_passant());
        assert!(r#move.is_double_pawn_push());
        assert!(!r#move.is_promotion());
        assert_eq!(r#move.is_castle_and_where(), None);
    }

    #[test]
    fn test_new_quiet_promotion() {
        let from = 10;
        let to = 57;
        let promoted_piece = KNIGHT;
        let r#move = Move::new_quiet_promotion(from, to, promoted_piece);
        assert_eq!(r#move.from(), from);
        assert_eq!(r#move.to(), to);
        assert_eq!(r#move.moving_piece(), PAWN);
        assert_eq!(r#move.promoted_piece(), promoted_piece);
        assert!(!r#move.is_capture());
        assert!(!r#move.is_capture_and_en_passant());
        assert!(!r#move.is_double_pawn_push());
        assert!(r#move.is_promotion());
        assert_eq!(r#move.is_castle_and_where(), None);
    }

    #[test]
    fn test_new_capture_promotion() {
        let from = 15;
        let to = 40;
        let promoted_piece = KNIGHT;
        let captured_piece = QUEEN;
        let r#move = Move::new_capture_promotion(from, to, promoted_piece, captured_piece);
        assert_eq!(r#move.from(), from);
        assert_eq!(r#move.to(), to);
        assert_eq!(r#move.moving_piece(), PAWN);
        assert_eq!(r#move.promoted_piece(), promoted_piece);
        assert_eq!(r#move.captured_piece(), captured_piece);
        assert!(r#move.is_capture());
        assert!(!r#move.is_capture_and_en_passant());
        assert!(!r#move.is_double_pawn_push());
        assert!(r#move.is_promotion());
        assert_eq!(r#move.is_castle_and_where(), None);
    }

}