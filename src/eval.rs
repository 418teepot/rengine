use std::cmp::min;

use crate::gamestate::{GameState, NUM_OF_PIECES, NUM_OF_PLAYERS, Side, KING, PAWN, ROOK, QUEEN, WHITE, BLACK, BISHOP, KNIGHT};
use crate::movegen::{KING_MOVES, rook_move_bitboard, bishop_move_bitboard, KNIGHT_MOVES, queen_move_bitboard};
use crate::search::Eval;

const PAWN_VALUE: Eval = 100;
const ROOK_VALUE: Eval = 500;
const KNIGHT_VALUE: Eval = 300;
const BISHOP_VALUE: Eval = 320;
const QUEEN_VALUE: Eval = 900;

const TEMPO_VALUE: Eval = 20;

pub static MATERIAL_VALUE: [Eval; NUM_OF_PIECES] = [PAWN_VALUE, ROOK_VALUE, KNIGHT_VALUE, BISHOP_VALUE, QUEEN_VALUE, KNIGHT_VALUE];

const PAWN_PHASE: i16 = 0;
const KNIGHT_PHASE: i16 = 1;
const BISHOP_PHASE: i16 = 1;
const ROOK_PHASE: i16 = 2;
const QUEEN_PHASE: i16 = 4;

pub static PHASE_WEIGHT: [i16; NUM_OF_PIECES] = [PAWN_PHASE, ROOK_PHASE, KNIGHT_PHASE, BISHOP_PHASE, QUEEN_PHASE, 0];

const TOTAL_PHASE: i16 = PAWN_PHASE * 16 + KNIGHT_PHASE * 4 + BISHOP_PHASE * 4 + ROOK_PHASE * 4 + QUEEN_PHASE * 2;

pub const NOT_CASTLED_PENALTY: Eval = -70;
pub const MISSING_PAWN_SHIELD_PENALTY: Eval = -50;

pub const MG_ROOK_MOBILITY_BONUS: Eval = 3;
pub const MG_BISHOP_MOBILITY_BONUS: Eval = 7;
pub const MG_KNIGHT_MOBILITY_BONUS: Eval = 7;
pub const MG_QUEEN_MOBILITY_BONUS: Eval = 2;

pub static PSQT_MG: [[[Eval; 64]; NUM_OF_PIECES]; NUM_OF_PLAYERS] = [
    [
        [
            0,   0,   0,   0,   0,   0,   0,   0,
            5,  10,  10, -20, -20,  10,  10,   5,
            5,  -5, -10,   0,   0, -10,  -5,   5,
            0,   0,  20,  20,  20,   0,   0,   0,
            5,   5,  10,  25,  25,  10,   5,   5,
            10,  10,  20,  30,  30,  20,  10,  10,
            50,  50,  50,  50,  50,  50,  50,  50,
            0,   0,   0,   0,   0,   0,   0,   0,
        ],
        [
            0,  0,  0,  5,  5,  0,  0,  0,
            -5,  0,  0,  0,  0,  0,  0, -5,
            -5,  0,  0,  0,  0,  0,  0, -5,
            -5,  0,  0,  0,  0,  0,  0, -5,
            -5,  0,  0,  0,  0,  0,  0, -5,
            -5,  0,  0,  0,  0,  0,  0, -5,
            5, 10, 10, 10, 10, 10, 10,  5,
            0,  0,  0,  0,  0,  0,  0,  0,
        ],
        [
            -50,-40,-30,-30,-30,-30,-40,-50,
            -40,-20,  0,  5,  5,  0,-20,-40,
            -30,  5, 10, 15, 15, 10,  5,-30,
            -30,  0, 15, 20, 20, 15,  0,-30,
            -30,  5, 15, 20, 20, 15,  5,-30,
            -30,  0, 10, 15, 15, 10,  0,-30,
            -40,-20,  0,  0,  0,  0,-20,-40,
            -50,-40,-30,-30,-30,-30,-40,-50,
        ],
        [
            -20,-10,-10,-10,-10,-10,-10,-20,
            -10,  5,  0,  0,  0,  0,  5,-10,
            -10, 10, 10, 10, 10, 10, 10,-10,
            -10,  0, 10, 10, 10, 10,  0,-10,
            -10,  5,  5, 10, 10,  5,  5,-10,
            -10,  0,  5, 10, 10,  5,  0,-10,
            -10,  0,  0,  0,  0,  0,  0,-10,
            -20,-10,-10,-10,-10,-10,-10,-20,
        ],
        [
            -20,-10,-10, -5, -5,-10,-10,-20,
            -10,  0,  5,  0,  0,  0,  0,-10,
            -10,  5,  5,  5,  5,  5,  0,-10,
            0,  0,  5,  5,  5,  5,  0, -5,
            -5,  0,  5,  5,  5,  5,  0, -5,
            -10,  0,  5,  5,  5,  5,  0,-10,
            -10,  0,  0,  0,  0,  0,  0,-10,
            -20,-10,-10, -5, -5,-10,-10,-20,
        ],
        [
            -20, 30, 20,-30, 0,-30,30,-20,
            -50,-50,-50,-50,-50,-50,-50,-50,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
        ],
    ], // White
    [
        [
            0,   0,   0,   0,   0,   0,   0,   0,
	50,  50,  50,  50,  50,  50,  50,  50,
	10,  10,  20,  30,  30,  20,  10,  10,
	5,   5,  10,  25,  25,  10,   5,   5,
	0,   0,  20,  20,  20,   0,   0,   0,
	5,  -5, -10,   0,   0, -10,  -5,   5,
	5,  10,  10, -20, -20,  10,  10,   5,
	0,   0,   0,   0,   0,   0,   0,   0
        ],
        [
            0,  0,  0,  0,  0,  0,  0,  0,
    5, 10, 10, 10, 10, 10, 10,  5,
	-5,  0,  0,  0,  0,  0,  0, -5,
	-5,  0,  0,  0,  0,  0,  0, -5,
	-5,  0,  0,  0,  0,  0,  0, -5,
	-5,  0,  0,  0,  0,  0,  0, -5,
	-5,  0,  0,  0,  0,  0,  0, -5,
	0,  0,  0,  5,  5,  0,  0,  0

        ],
        [
            -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  0,  0,  0,-20,-40,
	-30,  0, 10, 15, 15, 10,  0,-30,
	-30,  5, 15, 20, 20, 15,  5,-30,
	-30,  0, 15, 20, 20, 15,  0,-30,
	-30,  5, 10, 15, 15, 10,  5,-30,
	-40,-20,  0,  5,  5,  0,-20,-40,
	-50,-40,-30,-30,-30,-30,-40,-50,

        ],
        [
            -20,-10,-10,-10,-10,-10,-10,-20,
	-10,  0,  0,  0,  0,  0,  0,-10,
	-10,  0,  5, 10, 10,  5,  0,-10,
	-10,  5,  5, 10, 10,  5,  5,-10,
	-10,  0, 10, 10, 10, 10,  0,-10,
	-10, 10, 10, 10, 10, 10, 10,-10,
	-10,  5,  0,  0,  0,  0,  5,-10,
	-20,-10,-10,-10,-10,-10,-10,-20,
        ],
        [
            -20,-10,-10, -5, -5,-10,-10,-20,
	-10,  0,  0,  0,  0,  0,  0,-10,
	-10,  0,  5,  5,  5,  5,  0,-10,
	-5,  0,  5,  5,  5,  5,  0, -5,
	0,  0,  5,  5,  5,  5,  0, -5,
	-10,  5,  5,  5,  5,  5,  0,-10,
	-10,  0,  5,  0,  0,  0,  0,-10,
	-20,-10,-10, -5, -5,-10,-10,-20

        ],
        [
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            -50,-50,-50,-50,-50,-50,-50,-50,
            -20, 30, 20,-30, 0,-30,30,-20,
        ],

    ], // Black
];

pub static PSQT_EG: [[[Eval; 64]; NUM_OF_PIECES]; NUM_OF_PLAYERS] = 
[
    [
        [
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            5, 10,10,10,10,10,10,5,
            10,20,20,20,20,20,20,10,
            15,30,30,30,30,30,30,15,
            20,40,40,40,40,40,40,20,
            50,60,60,60,60,60,60,50,
            0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            -20, -10, -10, -10, -10, -10, -10, -20,
            -10, 0, 0, 0, 0, 0, 0, -10,
            -10, 0, 0, 0, 0, 0, 0, -10,
            -10, 0, 0, 0, 0, 0, 0, -10,
            -10, 0, 0, 0, 0, 0, 0, -10,
            -10, 0, 0, 0, 0, 0, 0, -10,
            -10, 0, 0, 0, 0, 0, 0, -10,
            -20, -10, -10, -10, -10, -10, -10, -20,
        ],
        [
            -30, -20, -20, -20, -20, -20, -20, -30,
            -20, 0, 0, 0, 0, 0, 0, -20,
            -20, 0, 0, 0, 0, 0, 0, -20,
            -20, 0, 0, 0, 0, 0, 0, -20,
            -20, 0, 0, 0, 0, 0, 0, -20,
            -20, 0, 0, 0, 0, 0, 0, -20,
            -20, 0, 0, 0, 0, 0, 0, -20,
            -30, -20, -20, -20, -20, -20, -20, -30,
        ],
        [
            -30, 0, 0, 0, 0, 0, 0, -30,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            -30, 0, 0, 0, 0, 0, 0, -30,
        ],
        [
            -10, 0, 0, 0, 0, 0, 0, -10,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            -10, 0, 0, 0, 0, 0, 0, -10,
        ],
        [
            -40, -30, -30, -30, -30, -30, -30, -40,
            -30, -10, -10, -10, -10, -10, -10, -30,
            -30, -10,   5,   5,   5,   5, -10, -30,
            -30, -10,   5,  10,  10,   5, -10, -30,
            -30, -10,   5,  10,  10,   5, -10, -30,
            -30, -10,   5,   0,   0,   5, -10, -30,
            -30, -10, -10, -10, -10, -10, -10, -30,
            -40, -30, -30, -30, -30, -30, -30, -40,
        ],
    ], // White
    [
        [
            0, 0, 0, 0, 0, 0, 0, 0,
            50,60,60,60,60,60,60,50,
            20,40,40,40,40,40,40,20,
            15,30,30,30,30,30,30,15,
            10,20,20,20,20,20,20,10,
            5, 10,10,10,10,10,10,5,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            -20, -10, -10, -10, -10, -10, -10, -20,
            -10, 0, 0, 0, 0, 0, 0, -10,
            -10, 0, 0, 0, 0, 0, 0, -10,
            -10, 0, 0, 0, 0, 0, 0, -10,
            -10, 0, 0, 0, 0, 0, 0, -10,
            -10, 0, 0, 0, 0, 0, 0, -10,
            -10, 0, 0, 0, 0, 0, 0, -10,
            -20, -10, -10, -10, -10, -10, -10, -20,
        ],
        [
            -30, -20, -20, -20, -20, -20, -20, -30,
            -20, 0, 0, 0, 0, 0, 0, -20,
            -20, 0, 0, 0, 0, 0, 0, -20,
            -20, 0, 0, 0, 0, 0, 0, -20,
            -20, 0, 0, 0, 0, 0, 0, -20,
            -20, 0, 0, 0, 0, 0, 0, -20,
            -20, 0, 0, 0, 0, 0, 0, -20,
            -30, -20, -20, -20, -20, -20, -20, -30,
        ],
        [
            -30, 0, 0, 0, 0, 0, 0, -30,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            -30, 0, 0, 0, 0, 0, 0, -30,
        ],
        [
            -10, 0, 0, 0, 0, 0, 0, -10,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            -10, 0, 0, 0, 0, 0, 0, -10,
        ],
        [
            -40, -30, -30, -30, -30, -30, -30, -40,
            -30, -10, -10, -10, -10, -10, -10, -30,
            -30, -10,   5,   5,   5,   5, -10, -30,
            -30, -10,   5,  10,  10,   5, -10, -30,
            -30, -10,   5,  10,  10,   5, -10, -30,
            -30, -10,   5,   0,   0,   5, -10, -30,
            -30, -10, -10, -10, -10, -10, -10, -30,
            -40, -30, -30, -30, -30, -30, -30, -40,
        ],
    ], // Black
];

impl GameState {
    pub fn static_eval(&self) -> Eval {
        let our_side = self.side_to_move();
        let enemy_side = our_side ^ 1;

        // Material Value
        let mg_eval = self.mg_eval(our_side, enemy_side);
        let eg_eval = self.eg_eval(our_side, enemy_side);
        let phase = self.phase();

        ((mg_eval * (256 - phase)) + (eg_eval * phase)) / 256
    }

    pub fn phase(&self) -> Eval {
        let mut phase = TOTAL_PHASE + self.phase;
        phase = (phase * 256 + (TOTAL_PHASE / 2)) / TOTAL_PHASE;
        phase as Eval
    }

    pub fn mg_eval(&self, our_side: Side, enemy_side: Side) -> Eval {
        (self.material[our_side] - self.material[enemy_side]) 
        + (self.psqt_mg[our_side] - self.psqt_mg[enemy_side]) 
        + self.king_safety_mg(our_side)
        + (self.mobility_mg(our_side) - self.mobility_mg(enemy_side))
        + TEMPO_VALUE
    }

    pub fn eg_eval(&self, our_side: Side, enemy_side: Side) -> Eval {
        (self.material[our_side] - self.material[enemy_side]) 
        + (self.psqt_eg[our_side] - self.psqt_eg[enemy_side]) 
        + TEMPO_VALUE
    }

    fn king_safety_mg(&self, our_side: Side) -> Eval {
        if !self.has_castled[our_side] {
            return NOT_CASTLED_PENALTY;
        } else {
            return (3 - min(3, (KING_MOVES[self.piece_boards[our_side][KING].next_piece_index()] & self.piece_boards[our_side][PAWN]).0.count_ones() as Eval)) * MISSING_PAWN_SHIELD_PENALTY;
        }
    }

    fn mobility_mg(&self, side: Side) -> Eval {
        let blockers = self.occupancy(WHITE) | self.occupancy(BLACK);
        let mut eval = 0; 
        for from_sq in self.piece_boards[side][ROOK] {
            eval += rook_move_bitboard(from_sq, blockers).0.count_ones() as Eval * MG_ROOK_MOBILITY_BONUS;
        }
        for from_sq in self.piece_boards[side][BISHOP] {
            eval += bishop_move_bitboard(from_sq, blockers).0.count_ones() as Eval * MG_BISHOP_MOBILITY_BONUS;
        }
        for from_sq in self.piece_boards[side][KNIGHT] {
            eval += KNIGHT_MOVES[from_sq].0.count_ones() as Eval * MG_KNIGHT_MOBILITY_BONUS;
        }
        for from_sq in self.piece_boards[side][QUEEN] {
            eval += queen_move_bitboard(from_sq, blockers).0.count_ones() as Eval * MG_QUEEN_MOBILITY_BONUS;
        }
        eval
    }

    fn tempo_value_mg(&self) -> Eval {
        TEMPO_VALUE
    }

    pub fn has_repitition(&self) -> bool {
        let mut index = 2;
        loop {
            let backtrace_ply: isize = self.plys as isize - (index * 2);
            if backtrace_ply < 0 {
                return false;
            }
            if self.history[backtrace_ply as usize].fifty_move_rule == 0 {
                return false;
            }
            if self.history[backtrace_ply as usize].zobrist == self.zobrist {
                return true;
            }            
            index += 1;
        }
    }
}
