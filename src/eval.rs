use std::cmp::min;

use crate::bitboard::Bitboard;
use crate::gamestate::{GameState, NUM_OF_PIECES, NUM_OF_PLAYERS, Side, KING, PAWN, ROOK, QUEEN, WHITE, BLACK, BISHOP, KNIGHT, Piece};
use crate::movegen::{KING_MOVES, rook_move_bitboard, bishop_move_bitboard, KNIGHT_MOVES, queen_move_bitboard, FILE_BITMASK};
use crate::r#move;
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
            -30, -10,   5,   5,   5,   5, -10, -30,
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
            -30, -10,   5,   5,   5,   5, -10, -30,
            -30, -10, -10, -10, -10, -10, -10, -30,
            -40, -30, -30, -30, -30, -30, -30, -40,
        ],
    ], // Black
];

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
        + self.king_safety_mg(our_side)
        // + (self.mobility_mg(our_side) - self.mobility_mg(enemy_side))
        + self.pieces_mg(our_side) - self.pieces_mg(enemy_side)
        + self.pawns_mg(our_side) - self.pawns_mg(enemy_side)
        + TEMPO_VALUE
    }

    fn pieces_mg(&self, our_side: Side) -> Eval {
        [0, 20, 50][self.rook_on_open_file(our_side) as usize]
    }

    fn pawns_mg(&self, our_side: Side) -> Eval {
        let mut eval = 0;
        for pawn in self.piece_boards[our_side][PAWN] {
            let file = pawn % 8;
            if (self.piece_boards[our_side][PAWN] & FILE_BITMASK[file]).0.count_ones() > 1 {
                eval -= 10;
            }
            if (self.piece_boards[our_side][PAWN] & ISOLATED_MASKS[file]).is_empty() {
                eval -= 6;
            }
        }
        eval
    }

    fn pawns_eg(&self, our_side: Side) -> Eval {
        let mut eval = 0;
        for pawn in self.piece_boards[our_side][PAWN] {
            let file = pawn % 8;
            if (self.piece_boards[our_side][PAWN] & FILE_BITMASK[file]).0.count_ones() > 1 {
                eval -= 30;
            }
            if (self.piece_boards[our_side][PAWN] & ISOLATED_MASKS[file]).is_empty() {
                eval -= 15;
            }
        }
        eval
    }

    pub fn eg_eval(&self, our_side: Side, enemy_side: Side) -> Eval {
        (self.material[our_side] - self.material[enemy_side]) 
        + (self.psqt_eg[our_side] - self.psqt_eg[enemy_side])
        + self.pieces_eg(our_side) - self.pieces_eg(enemy_side)
        // + self.mobility_eg(our_side) - self.mobility_eg(enemy_side) 
        + self.pawns_eg(our_side) - self.pawns_eg(enemy_side)
        + TEMPO_VALUE
    }

    fn pieces_eg(&self, our_side: Side) -> Eval {
        [0, 10, 30][self.rook_on_open_file(our_side) as usize]
    }

    fn king_safety_mg(&self, our_side: Side) -> Eval {
        if !self.has_castled[our_side] {
            NOT_CASTLED_PENALTY
        } else {
            (3 - min(3, (KING_MOVES[self.piece_boards[our_side][KING].next_piece_index()] & self.piece_boards[our_side][PAWN]).0.count_ones() as Eval)) * MISSING_PAWN_SHIELD_PENALTY
        }
    }

    fn rook_on_open_file(&self, our_side: Side) -> u8 {
        let mut rooks = 0;
        for file in 0..8 {
            if (FILE_BITMASK[file] & self.piece_boards[our_side][ROOK]).is_empty() {
                if (FILE_BITMASK[file] & self.piece_boards[our_side][ROOK]).is_filled() {
                    rooks += 1;
                }
            }
        }
        std::cmp::max(2, rooks)
    }

    fn mobility_mg(&self, side: Side) -> Eval {
        let mut eval = 0;
        let blockers = self.occupancy(WHITE) | self.occupancy(BLACK);
        let king_square = self.piece_boards[side][KING].next_piece_index();
        let enemy_side = side ^ 1;
        let pinned_hv = self.get_hv_pinmask(king_square, blockers, enemy_side);
        let pinned_d12 = self.get_diagonal_pinmask(king_square, blockers, enemy_side);
        for from_sq in self.piece_boards[side][ROOK] & !pinned_d12 {
            eval += MG_ROOK_MOBILITY_BONUS[rook_move_bitboard(from_sq, blockers).0.count_ones() as usize];
        }
        for from_sq in self.piece_boards[side][BISHOP] & !pinned_hv {
            eval += MG_BISHOP_MOBILITY_BONUS[bishop_move_bitboard(from_sq, blockers).0.count_ones() as usize];
        }
        for from_sq in self.piece_boards[side][KNIGHT] & !(pinned_hv | pinned_d12) {
            eval += MG_KNIGHT_MOBILITY_BONUS[KNIGHT_MOVES[from_sq].0.count_ones() as usize];
        }
        for from_sq in self.piece_boards[side][QUEEN] & !(pinned_d12 | pinned_hv) {
            eval += MG_QUEEN_MOBILITY_BONUS[queen_move_bitboard(from_sq, blockers).0.count_ones() as usize];
        }
        eval
    }

    pub fn mobility_eg(&self, side: Side) -> Eval {
        let mut eval = 0;
        let blockers = self.occupancy(WHITE) | self.occupancy(BLACK);
        let king_square = self.piece_boards[side][KING].next_piece_index();
        let enemy_side = side ^ 1;
        let pinned_hv = self.get_hv_pinmask(king_square, blockers, enemy_side);
        let pinned_d12 = self.get_diagonal_pinmask(king_square, blockers, enemy_side);
        for from_sq in self.piece_boards[side][ROOK] & !pinned_d12 {
            eval += EG_ROOK_MOBILITY_BONUS[rook_move_bitboard(from_sq, blockers).0.count_ones() as usize];
        }
        for from_sq in self.piece_boards[side][BISHOP] & !pinned_hv {
            eval += EG_BISHOP_MOBILITY_BONUS[bishop_move_bitboard(from_sq, blockers).0.count_ones() as usize];
        }
        for from_sq in self.piece_boards[side][KNIGHT] & !(pinned_hv | pinned_d12) {
            eval += EG_KNIGHT_MOBILITY_BONUS[KNIGHT_MOVES[from_sq].0.count_ones() as usize];
        }
        for from_sq in self.piece_boards[side][QUEEN] & !(pinned_d12 | pinned_hv) {
            eval += EG_QUEEN_MOBILITY_BONUS[queen_move_bitboard(from_sq, blockers).0.count_ones() as usize];
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

    pub fn game_is_over(&mut self) -> bool {
        let moves = self.generate_pseudo_legal_moves();
        for r#move in moves {
            if self.apply_pseudo_legal_move(r#move) {
                self.undo_move();
                return false;
            }
        }
        true
    }
}
