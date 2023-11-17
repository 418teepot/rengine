use std::cmp::{min, max};

use crate::bitboard::{Bitboard, Square, NUM_OF_SQUARES};
use crate::gamestate::{GameState, NUM_OF_PIECES, NUM_OF_PLAYERS, Side, KING, PAWN, ROOK, QUEEN, WHITE, BLACK, BISHOP, KNIGHT};
use crate::movegen::{KING_MOVES, rook_move_bitboard, bishop_move_bitboard, KNIGHT_MOVES, queen_move_bitboard, FILE_BITMASK, RANK_BITMASK};
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
pub const MISSING_PAWN_SHIELD_PENALTY: Eval = -70;

pub static MG_ROOK_MOBILITY_BONUS: [Eval; 15] = [-60,-20,2,3,3,11,22,31,40,40,41,48,57,57,62];
pub static MG_BISHOP_MOBILITY_BONUS: [Eval; 14] = [-48,-20,16,26,38,51,55,63,63,68,81,81,91,98];
pub static MG_KNIGHT_MOBILITY_BONUS: [Eval; 9] = [-62,-53,-12,-4,3,13,22,28,33];
pub static MG_QUEEN_MOBILITY_BONUS: [Eval; 28] = [-30,-12,-8,-9,20,23,23,35,38,53,64,65,65,66,67,67,72,72,77,79,93,108,108,108,110,114,114,116];

pub static EG_KNIGHT_MOBILITY_BONUS: [Eval; 9] = [-81,-56,-31,-16,5,11,17,20,25];
pub static EG_BISHOP_MOBILITY_BONUS: [Eval; 14] = [-59,-23,-3,13,24,42,54,57,65,73,78,86,88,97];
pub static EG_ROOK_MOBILITY_BONUS: [Eval; 15] = [-78,-17,23,39,70,99,103,121,134,139,158,164,168,169,172];
pub static EG_QUEEN_MOBILITY_BONUS: [Eval; 28] = [-48,-30,-7,19,40,55,59,75,78,96,96,100,121,127,131,133,136,141,147,150,151,168,168,171,182,182,192,219];

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
            -30,  5, 10, 15, 15, 15,  5,-30,
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
	-30,  5, 10, 15, 15, 15,  5,-30,
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
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
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
            -30, -10,   5,   5,   5,   5, -10, -30,
            -30, -10, -10, -10, -10, -10, -10, -30,
            -40, -30, -30, -30, -30, -30, -30, -40,
        ],
    ], // White
    [
        [
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
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
            -30, -10,   5,   5,   5,   5, -10, -30,
            -30, -10, -10, -10, -10, -10, -10, -30,
            -40, -30, -30, -30, -30, -30, -30, -40,
        ],
    ], // Black
];

// static attack_weight: [Eval; 7] = [0, 50, 75, 88, 94, 97, 99];

lazy_static! {
    static ref ISOLATED_MASKS: [Bitboard; 8] = {
        let mut boards = [Bitboard(0); 8];
        boards[0] = FILE_BITMASK[1];
        boards[7] = FILE_BITMASK[6];
        for file in 1..7 {
            boards[file] = FILE_BITMASK[file + 1] | FILE_BITMASK[file - 1];
        }
        boards
    };

    static ref PASSED_MASK: [[Bitboard; NUM_OF_SQUARES]; NUM_OF_PLAYERS] = {
        let mut masks = [[Bitboard(0); 64]; 2];
        // White
        for square in 8..56 {
            let file: isize = square as isize % 8;
            let file_mask = FILE_BITMASK[file as usize];
            let file_mask_left = FILE_BITMASK[max(0, file - 1) as usize];
            let file_mask_right = FILE_BITMASK[min(7, file + 1) as usize];
            let triple_file_mask = file_mask | file_mask_left | file_mask_right;
            let rank = square / 8;
            let forward_mask_white = Bitboard::full() << (8 * (rank + 1));
            let forward_mask_black = Bitboard::full() >> (8 * (rank - 1));
            masks[WHITE][square] = forward_mask_white & triple_file_mask;
            masks[BLACK][square] = forward_mask_black & triple_file_mask;
        }

        masks
    };
}

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
        + (self.king_safety_mg(our_side) - self.king_safety_mg(enemy_side))
        + (self.mobility_mg(our_side) - self.mobility_mg(enemy_side))
        + (self.pieces_mg(our_side) - self.pieces_mg(enemy_side))
        + (self.pawns_mg(our_side) - self.pawns_mg(enemy_side))
        + (self.space_mg(our_side) - self.space_mg(enemy_side))
        + TEMPO_VALUE
    }

    fn pieces_mg(&self, our_side: Side) -> Eval {
        [0, 20, 50][self.rook_on_open_file(our_side) as usize]
    }

    fn pawns_mg(&self, our_side: Side) -> Eval {
        let mut eval = 0;
        for pawn in self.piece_boards[our_side][PAWN] {
            if self.is_doubled(our_side, pawn) {
                eval -= 10;
            }
            if self.is_isolated(our_side, pawn) {
                eval -= 6;
            }
            if self.is_passed(our_side, pawn) {
                eval += [0, 10, 20, 30, 60, 160, 280, 0][Self::ranked_passed_pawn(our_side, pawn)];
            }
            let is_connected = self.is_connected(our_side, pawn);
            if is_connected > 0 {
                eval += is_connected as Eval * 20;
            }
        }
        eval
    }

    fn ranked_passed_pawn(side: Side, square: Square) -> usize {
        if side == WHITE {
            square / 8
        } else {
            8 - (square / 8)
        }
    }

    fn pawns_eg(&self, our_side: Side) -> Eval {
        let mut eval = 0;
        for pawn in self.piece_boards[our_side][PAWN] {
            if self.is_doubled(our_side, pawn) {
                eval -= 30;
            }
            if self.is_isolated(our_side, pawn) {
                eval -= 15;
            }
            if self.is_passed(our_side, pawn) {
                eval += [0, 30, 35, 45, 70, 150, 280, 0][Self::ranked_passed_pawn(our_side, pawn)];
            }
            let is_connected = self.is_connected(our_side, pawn);
            if is_connected > 0 {
                eval += is_connected as Eval * 20;
            }
        }
        eval
    }

    fn is_isolated(&self, our_side: Side, square: Square) -> bool {
        (self.piece_boards[our_side][PAWN] & ISOLATED_MASKS[square % 8]).is_empty()
    }

    fn is_doubled(&self, our_side: Side, square: Square) -> bool {
        (self.piece_boards[our_side][PAWN] & FILE_BITMASK[square % 8]).0.count_ones() > 1
    }

    fn is_passed(&self, our_side: Side, square: Square) -> bool {
        (self.piece_boards[our_side ^ 1][PAWN] & PASSED_MASK[our_side][square]).is_empty()
    }

    fn is_connected(&self, our_side: Side, square: Square) -> u8 {
        let pawn_board = Bitboard::square(square);
        let attacks = if our_side == WHITE {
            ((pawn_board &!FILE_BITMASK[0]) >> 7) 
            | ((pawn_board &!FILE_BITMASK[7]) >> 9) 
        } else {
            ((pawn_board &!FILE_BITMASK[0]) << 7)
            | ((pawn_board &!FILE_BITMASK[7]) << 9)
        };
        (self.piece_boards[our_side][PAWN] & attacks).0.count_ones() as u8
    }

    pub fn eg_eval(&self, our_side: Side, enemy_side: Side) -> Eval {
        (self.material[our_side] - self.material[enemy_side]) 
        + (self.psqt_eg[our_side] - self.psqt_eg[enemy_side])
        + (self.pieces_eg(our_side) - self.pieces_eg(enemy_side))
        + (self.mobility_eg(our_side) - self.mobility_eg(enemy_side)) 
        + (self.pawns_eg(our_side) - self.pawns_eg(enemy_side))
        + TEMPO_VALUE
    }

    fn pieces_eg(&self, our_side: Side) -> Eval {
        [0, 10, 30][self.rook_on_open_file(our_side) as usize]
    }

    fn king_safety_mg(&self, our_side: Side) -> Eval {
        let mut eval = 0;
        if !self.has_castled[our_side] {
            eval += NOT_CASTLED_PENALTY
        } else {
            eval += (3 - min(3, (KING_MOVES[self.piece_boards[our_side][KING].next_piece_index()] & self.piece_boards[our_side][PAWN]).0.count_ones() as Eval)) * MISSING_PAWN_SHIELD_PENALTY
        }
        eval
    }

    pub fn space_mg(&self, our_side: Side) -> Eval {
        let space_area = self.space_area(our_side);
        let open_file_count = self.open_file_count();
        let piece_count = self.our_piece_count(our_side);
        let weight = piece_count - (2 * open_file_count);
        (space_area * weight) as Eval
    }

    pub fn open_file_count(&self) -> u8 {
        let mut files = 0;
        let pawns = self.piece_boards[WHITE][PAWN] | self.piece_boards[BLACK][PAWN];
        for file in 0..8 {
            if (pawns & FILE_BITMASK[file]).is_empty() {
                files += 1;
            }
        }
        files
    }

    pub fn our_piece_count(&self, our_side: Side) -> u8 {
        (self.piece_boards[our_side][ROOK].0.count_ones()
        + self.piece_boards[our_side][KNIGHT].0.count_ones()
        + self.piece_boards[our_side][BISHOP].0.count_ones()
        + self.piece_boards[our_side][QUEEN].0.count_ones()) as u8
    }

    pub fn space_area(&self, our_side: Side) -> u8 {
        let central_files = FILE_BITMASK[2] | FILE_BITMASK[3] | FILE_BITMASK[4] | FILE_BITMASK[5];
        let enemy_side = our_side ^ 1;
        let mut space_total = 0;
        if our_side == WHITE {
            let our_ranks = RANK_BITMASK[1] | RANK_BITMASK[2] |RANK_BITMASK[3];
            let space_area = central_files & our_ranks;
            let pawn_attacks = ((self.piece_boards[enemy_side][PAWN] & !FILE_BITMASK[0]) >> 7)
            & ((self.piece_boards[enemy_side][PAWN] & !FILE_BITMASK[7]) >> 9);
            let space_area = space_area & !pawn_attacks & !self.piece_boards[our_side][PAWN];
            space_total += space_area.0.count_ones() as u8;
            let safe_behind_pawn_one_square = (self.piece_boards[our_side][PAWN] >> 8) & space_area;
            space_total += safe_behind_pawn_one_square.0.count_ones() as u8;
            let safe_behind_pawn_two_square = (self.piece_boards[our_side][PAWN] >> 16) & space_area;
            space_total += safe_behind_pawn_two_square.0.count_ones() as u8;
            let safe_behind_pawn_three_square = (self.piece_boards[our_side][PAWN] >> 24) & space_area;
            space_total += safe_behind_pawn_three_square.0.count_ones() as u8;
        } else {
            let our_ranks = RANK_BITMASK[4] | RANK_BITMASK[5] |RANK_BITMASK[6];
            let space_area = central_files & our_ranks;
            let pawn_attacks = ((self.piece_boards[enemy_side][PAWN] & !FILE_BITMASK[0]) << 7)
            & ((self.piece_boards[enemy_side][PAWN] & !FILE_BITMASK[7]) << 9);
            let space_area = space_area & !pawn_attacks & !self.piece_boards[our_side][PAWN];
            space_total += space_area.0.count_ones() as u8;
            let safe_behind_pawn_one_square = (self.piece_boards[our_side][PAWN] << 8) & space_area;
            space_total += safe_behind_pawn_one_square.0.count_ones() as u8;
            let safe_behind_pawn_two_square = (self.piece_boards[our_side][PAWN] << 16) & space_area;
            space_total += safe_behind_pawn_two_square.0.count_ones() as u8;
            let safe_behind_pawn_three_square = (self.piece_boards[our_side][PAWN] << 24) & space_area;
            space_total += safe_behind_pawn_three_square.0.count_ones() as u8;
        }
        space_total
    }

    fn rook_on_open_file(&self, our_side: Side) -> u8 {
        let mut rooks = 0;
        for file in 0..8 {
            if (FILE_BITMASK[file] & self.piece_boards[our_side][ROOK]).is_empty()
                && (FILE_BITMASK[file] & self.piece_boards[our_side][ROOK]).is_filled() {
                    rooks += 1;
            }
        }
        std::cmp::max(2, rooks)
    }

    fn mobility_mg(&self, side: Side) -> Eval {
        let mut eval = 0;
        let blockers = self.occupancy(WHITE) | self.occupancy(BLACK);
        let mobility_area = self.mobility_area(side);
        for from_sq in self.piece_boards[side][ROOK] {
            eval += MG_ROOK_MOBILITY_BONUS[(rook_move_bitboard(from_sq, blockers) & mobility_area).0.count_ones() as usize];
        }
        for from_sq in self.piece_boards[side][BISHOP] {
            eval += MG_BISHOP_MOBILITY_BONUS[(bishop_move_bitboard(from_sq, blockers) & mobility_area).0.count_ones() as usize];
        }
        for from_sq in self.piece_boards[side][KNIGHT] {
            eval += MG_KNIGHT_MOBILITY_BONUS[(KNIGHT_MOVES[from_sq] & mobility_area).0.count_ones() as usize];
        }
        for from_sq in self.piece_boards[side][QUEEN] {
            eval += MG_QUEEN_MOBILITY_BONUS[(queen_move_bitboard(from_sq, blockers) & mobility_area).0.count_ones() as usize];
        }
        eval
    }

    pub fn mobility_eg(&self, side: Side) -> Eval {
        let mut eval = 0;
        let blockers = self.occupancy(WHITE) | self.occupancy(BLACK);
        let mobility_area = self.mobility_area(side);
        for from_sq in self.piece_boards[side][ROOK] {
            eval += EG_ROOK_MOBILITY_BONUS[(rook_move_bitboard(from_sq, blockers) & mobility_area).0.count_ones() as usize];
        }
        for from_sq in self.piece_boards[side][BISHOP] {
            eval += EG_BISHOP_MOBILITY_BONUS[(bishop_move_bitboard(from_sq, blockers) & mobility_area).0.count_ones() as usize];
        }
        for from_sq in self.piece_boards[side][KNIGHT] {
            eval += EG_KNIGHT_MOBILITY_BONUS[(KNIGHT_MOVES[from_sq] & mobility_area).0.count_ones() as usize];
        }
        for from_sq in self.piece_boards[side][QUEEN] {
            eval += EG_QUEEN_MOBILITY_BONUS[(queen_move_bitboard(from_sq, blockers) & mobility_area).0.count_ones() as usize];
        }

        eval
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

    pub fn calculate_movetime(&self, wtime: u64, btime: u64) -> u64 {
        /* 
        f(x) = ax + b
        f(0) = 40
        40 = a * 0 + b
        b = 40
        f(256) = 5
        5 = a * 256 + 40 | -40
        -35 = a * 256    | / 256
        -35/256 = a
        */
        (if self.side_to_move() == WHITE {
            wtime
        } else {
            btime
        }) / self.moves_left() as u64
    }

    fn moves_left(&self) -> u16 {
        ((self.phase() as f64) * (-35.0_f64 / 256.0_f64) + 40.0_f64) as u16
    }

    fn mobility_area(&self, side: Side) -> Bitboard {
        let mut area = Bitboard::full();
        let enemy_side = side ^ 1;
        area &= !(if side == WHITE {
            ((self.piece_boards[enemy_side][PAWN] & !FILE_BITMASK[7]) >> 7)
            | ((self.piece_boards[enemy_side][PAWN] & !FILE_BITMASK[0]) >> 9)
        } else {
            ((self.piece_boards[enemy_side][PAWN] & !FILE_BITMASK[0]) << 7)
            | ((self.piece_boards[enemy_side][PAWN] & !FILE_BITMASK[7]) << 9)
        });
        area &= !(self.piece_boards[side][PAWN] & 
            (if side == WHITE {
                RANK_BITMASK[1] | RANK_BITMASK[2]
            } else {
                RANK_BITMASK[6] | RANK_BITMASK[5]
            })
        );

        area
    }
}
