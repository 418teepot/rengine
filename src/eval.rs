use std::cell::SyncUnsafeCell;
use std::cmp::{min, max};

use crate::bitboard::{Bitboard, Square, NUM_OF_SQUARES};
use crate::gamestate::{GameState, NUM_OF_PIECES, NUM_OF_PLAYERS, Side, KING, PAWN, ROOK, QUEEN, WHITE, BLACK, BISHOP, KNIGHT};
use crate::movegen::{KING_MOVES, rook_move_bitboard, bishop_move_bitboard, KNIGHT_MOVES, queen_move_bitboard, FILE_BITMASK, RANK_BITMASK, knight_move_bitboard};
use crate::smpsearch::{Eval, AB_BOUND};

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
        // Is material draw?
        if self.piece_boards[WHITE][PAWN].is_empty() && self.piece_boards[BLACK][PAWN].is_empty() {
            if let Some(side) = self.knight_bishop_lonesome_side() {
                if side == enemy_side {
                    return 20 - self.bishop_knight_ending_eval(side);
                } else {
                    return -20 + self.bishop_knight_ending_eval(side);
                }
            }

            if self.is_material_draw() {
                return 0;
            }
        }

        // Material Value
        let mut mg_eval = self.mg_eval(our_side, enemy_side);
        let mut eg_eval = self.eg_eval(our_side, enemy_side);
        let blockers = self.occupancy(our_side) | self.occupancy(enemy_side);
        let mobility_us = self.mobility(our_side, self.mobility_area(our_side), blockers);
        let mobilits_enemy = self.mobility(enemy_side, self.mobility_area(enemy_side), blockers);
        mg_eval += mobility_us.0 - mobilits_enemy.0;
        eg_eval += mobility_us.1 - mobilits_enemy.1;
        let pawns_us = self.pawns(our_side);
        let pawns_enemy = self.pawns(enemy_side);
        mg_eval += pawns_us.0 - pawns_enemy.0;
        eg_eval += pawns_us.1 - pawns_enemy.1;
        // mg_eval += self.king_safety_mg(our_side) - self.king_safety_mg(enemy_side);
        let phase = self.phase();

        ((mg_eval * (256 - phase)) + (eg_eval * phase)) / 256
    }

    pub fn bishop_knight_ending_eval(&self, lonesome_side: Side) -> Eval {
        let king_square = self.piece_boards[lonesome_side][KING].next_piece_index() as i64;
        let bishop_square = self.piece_boards[lonesome_side ^ 1][BISHOP].next_piece_index() as i64;
        let manhatten_distance = self.manhatten_distance_corner_bishop(bishop_square, king_square);
        manhatten_distance * 100
    }

    pub fn manhatten_distance_corner_bishop(&self, bishop_square: i64, king_square: i64) -> Eval {
        let b: i64 = -1879048192 * bishop_square >> 31;
        let k: i64 = (king_square>>3) + ((king_square^b) & 7);
        let k: i64 = (15*(k>>3)^k)-(k>>3);
        k as Eval
    }

    fn pawns(&self, our_side: Side) -> (Eval, Eval) {
        let mut pawns_mg = 0;
        let mut pawns_eg = 0;
        for pawn in self.piece_boards[our_side][PAWN] {
            if (PASSED_MASK[our_side][pawn] & self.piece_boards[our_side ^ 1][PAWN]).is_empty() {
                let passed_rank = if our_side == WHITE {
                    pawn / 8
                } else {
                    8 - (pawn / 8)
                };
                unsafe {
                    pawns_mg += EVAL_PARAMS.mg_passed[passed_rank];
                    pawns_eg += EVAL_PARAMS.eg_passed[passed_rank];
                }
            }
            let file = pawn % 8;
            let isolated: bool = (ISOLATED_MASKS[file] & self.piece_boards[our_side][PAWN]).is_empty();
            let doubled: bool = {
                let pawn_bitboard = Bitboard(1 << pawn);
                let pawn_up_one = if our_side == WHITE {
                    pawn_bitboard << 8
                } else {
                    pawn_bitboard >> 8
                };
                (pawn_up_one & self.piece_boards[our_side][PAWN]).is_filled()
            };
            if doubled && isolated {
                unsafe {
                    pawns_mg -= EVAL_PARAMS.mg_doubled_isolated_penalty;
                    pawns_eg -= EVAL_PARAMS.eg_doubled_isolated_penalty;
                }
            } else if isolated {
                unsafe {
                    pawns_mg -= EVAL_PARAMS.mg_isolated_penalty;
                    pawns_eg -= EVAL_PARAMS.eg_isolated_penalty;
                }
            }
            if doubled {
                unsafe {
                    pawns_mg -= EVAL_PARAMS.mg_doubled_penalty;
                    pawns_eg -= EVAL_PARAMS.eg_doubled_penalty;
                }
            }
        }
        (pawns_mg, pawns_eg)
    }

    pub fn is_material_draw(&self) -> bool {
        if self.piece_boards[WHITE][ROOK].is_empty() && self.piece_boards[BLACK][ROOK].is_empty() && self.piece_boards[WHITE][QUEEN].is_empty() && self.piece_boards[BLACK][QUEEN].is_empty() {
            if self.piece_boards[BLACK][BISHOP].is_empty() && self.piece_boards[WHITE][BISHOP].is_empty() {
                if self.piece_boards[WHITE][KNIGHT].0.count_ones() < 3 && self.piece_boards[BLACK][KNIGHT].0.count_ones() < 3 { return true }
            } else if self.piece_boards[WHITE][KNIGHT].is_empty() && self.piece_boards[BLACK][KNIGHT].is_empty() {
               if (self.piece_boards[WHITE][BISHOP].0.count_ones() as i32 - self.piece_boards[BLACK][BISHOP].0.count_ones() as i32).abs() < 2 { return true }
            } else if ((self.piece_boards[WHITE][KNIGHT].0.count_ones() < 3 && self.piece_boards[WHITE][BISHOP].is_empty()) || (self.piece_boards[WHITE][BISHOP].0.count_ones() == 1 && self.piece_boards[WHITE][KNIGHT].is_empty())) && ((self.piece_boards[BLACK][KNIGHT].0.count_ones() < 3 && self.piece_boards[BLACK][BISHOP].is_empty()) || (self.piece_boards[BLACK][BISHOP].0.count_ones() == 1 && self.piece_boards[BLACK][KNIGHT].is_empty())) { return true }
          } else if self.piece_boards[WHITE][QUEEN].is_empty() && self.piece_boards[BLACK][QUEEN].is_empty() {
              if self.piece_boards[WHITE][ROOK].0.count_ones() == 1 && self.piece_boards[BLACK][ROOK].0.count_ones() == 1 {
                  if (self.piece_boards[WHITE][KNIGHT].0.count_ones() + self.piece_boards[WHITE][BISHOP].0.count_ones()) < 2 && (self.piece_boards[BLACK][KNIGHT].0.count_ones() + self.piece_boards[BLACK][BISHOP].0.count_ones()) < 2 { return true }
              } else if self.piece_boards[WHITE][ROOK].0.count_ones() == 1 && self.piece_boards[BLACK][ROOK].is_empty() {
                  if (self.piece_boards[WHITE][KNIGHT].0.count_ones() + self.piece_boards[WHITE][BISHOP].0.count_ones() == 0) && (((self.piece_boards[BLACK][KNIGHT].0.count_ones() + self.piece_boards[BLACK][BISHOP].0.count_ones()) == 1) || ((self.piece_boards[BLACK][KNIGHT].0.count_ones() + self.piece_boards[BLACK][BISHOP].0.count_ones()) == 2)) { return true }
              } else if (self.piece_boards[BLACK][ROOK].0.count_ones() == 1 && self.piece_boards[WHITE][ROOK].is_empty()) && ((self.piece_boards[BLACK][KNIGHT].0.count_ones() + self.piece_boards[BLACK][BISHOP].0.count_ones() == 0) && (((self.piece_boards[WHITE][KNIGHT].0.count_ones() + self.piece_boards[WHITE][BISHOP].0.count_ones()) == 1) || ((self.piece_boards[WHITE][KNIGHT].0.count_ones() + self.piece_boards[WHITE][BISHOP].0.count_ones()) == 2))) { return true }
          }
        false
    }

    pub fn knight_bishop_lonesome_side(&self) -> Option<Side> {
        if self.material[WHITE] == 0 && self.piece_boards[BLACK][ROOK].piece_count() == 0 && self.piece_boards[BLACK][QUEEN].piece_count() == 0
        && self.piece_boards[BLACK][BISHOP].piece_count() == 1 && self.piece_boards[BLACK][KNIGHT].piece_count() == 1 {
            return Some(WHITE);
        }
        if self.material[BLACK] == 0 && self.piece_boards[WHITE][ROOK].piece_count() == 0 && self.piece_boards[WHITE][QUEEN].piece_count() == 0 
        && self.piece_boards[WHITE][BISHOP].piece_count() == 1 && self.piece_boards[WHITE][KNIGHT].piece_count() == 1 {
            return Some(BLACK);
        }
        None
    }

    fn mg_eval(&self, our_side: Side, enemy_side: Side) -> Eval {
        (self.material_mg(our_side) - self.material_mg(enemy_side))
        + (self.psqt_mg(our_side) - self.psqt_mg(enemy_side))
    }

    fn material_mg(&self, our_side: Side) -> Eval {
        self.material[our_side]
    }

    fn psqt_mg(&self, our_side: Side) -> Eval {
        self.psqt_mg[our_side]
    }

    fn king_safety_mg(&self, our_side: Side) -> Eval {
        let king_square = self.piece_boards[our_side][KING];
        let king_square_index = king_square.next_piece_index();
        let king_file = king_square_index % 8;
        let mut eval = 0;
        for file_around_king in max(king_file - 1, 0)..min(king_file, 8) {
            if (FILE_BITMASK[file_around_king] & self.piece_boards[our_side][PAWN]).is_empty() {
                unsafe {
                    eval -= EVAL_PARAMS.open_king_file_punish_mg;
                }
            }
        }
        eval
    }   

    fn mobility(&self, our_side: Side, mobility_area: Bitboard, blockers: Bitboard) -> (Eval, Eval) {
        let mut mg_eval = 0;
        let mut eg_eval = 0;

        let defended_by_minors = self.defended_by_minors(our_side ^ 1, blockers);
        
        for piece in self.piece_boards[our_side][ROOK] {
            let moves = rook_move_bitboard(piece, blockers);
            let mobile_moves = moves & mobility_area & !defended_by_minors;
            let mobile_move_count = mobile_moves.0.count_ones() as usize;
            unsafe {
                mg_eval += EVAL_PARAMS.mg_rook_mobility[mobile_move_count];
                eg_eval += EVAL_PARAMS.eg_rook_mobility[mobile_move_count];
            }
        }

        for piece in self.piece_boards[our_side][QUEEN] {
            let moves = queen_move_bitboard(piece, blockers);
            let mobile_moves = moves & mobility_area & !defended_by_minors;
            let mobile_move_count = mobile_moves.0.count_ones() as usize;
            unsafe {
                mg_eval += EVAL_PARAMS.mg_queen_mobility[mobile_move_count];
                eg_eval += EVAL_PARAMS.eg_queen_mobility[mobile_move_count];
            }
        }

        for piece in self.piece_boards[our_side][BISHOP] {
            let moves = bishop_move_bitboard(piece, blockers);
            let mobile_moves = moves & mobility_area;
            let mobile_move_count = mobile_moves.0.count_ones() as usize;
            unsafe {
                mg_eval += EVAL_PARAMS.mg_bishop_mobility[mobile_move_count];
                eg_eval += EVAL_PARAMS.eg_bishop_mobility[mobile_move_count];
            }
        }

        for piece in self.piece_boards[our_side][KNIGHT] {
            let moves = knight_move_bitboard(piece);
            let mobile_moves = moves & mobility_area;
            let mobile_move_count = mobile_moves.0.count_ones() as usize;
            unsafe {
                mg_eval += EVAL_PARAMS.mg_knight_mobility[mobile_move_count];
                eg_eval += EVAL_PARAMS.eg_knight_mobility[mobile_move_count];
            }
        }

        (mg_eval, eg_eval)
    }

    fn defended_by_minors(&self, our_side: Side, blockers: Bitboard) -> Bitboard {
        let mut defended = Bitboard(0);
        for piece in self.piece_boards[our_side][KNIGHT] {
            defended |= knight_move_bitboard(piece);
        }

        for piece in self.piece_boards[our_side][BISHOP] {
            defended |= bishop_move_bitboard(piece, blockers);
        }
        
        defended
    } 

    fn eg_eval(&self, our_side: Side, enemy_side: Side) -> Eval {
        (self.material_eg(our_side) - self.material_eg(enemy_side))
        + (self.psqt_eg(our_side) - self.psqt_eg(enemy_side))
    }

    fn material_eg(&self, our_side: Side) -> Eval {
        self.material_eg[our_side]
    }

    fn psqt_eg(&self, our_side: Side) -> Eval {
        self.psqt_eg[our_side]
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

    pub fn phase(&self) -> Eval {
        let mut phase = TOTAL_PHASE + self.phase;
        phase = (phase * 256 + (TOTAL_PHASE / 2)) / TOTAL_PHASE;
        phase as Eval
    }

    pub fn has_repitition(&self) -> bool {
        for index in (0..self.history.len()).rev() {
            if self.history[index].zobrist == self.zobrist {
                return true;
            }
        }
        false
    }

    pub fn calculate_movetime(&self, wtime: u64, btime: u64, winc: u64, binc: u64) -> u64 {
        let time_left_move = if self.side_to_move() == WHITE {
            self.calculate_movetime_simple_side(wtime as i64, winc as i64)
        } else {
            self.calculate_movetime_simple_side(btime as i64, binc as i64)
        };
        time_left_move
    }

    pub fn calculate_movetime_simple_side(&self, time_left: i64, inc: i64) -> u64 {
        let mut time_left_move = time_left / 40 + inc / 2;
        if time_left_move >= time_left {
            time_left_move = time_left - 500;
        }

        if time_left_move < 0 {
            time_left_move = inc / 2;
        }
        time_left_move as u64
    }

    fn midgame_scale(&self) -> f64 {
        let scale =  -0.00006_f64 * ((self.phase() as f64 - 64_f64) * (self.phase() as f64 - 64_f64)) + 1.4_f64;
        if scale < 0.6_f64 {
            0.6_f64
        } else {
            scale
        }
    }

    fn moves_left(&self) -> u16 {
        ((self.phase() as f64) * (-40.0_f64 / 256.0_f64) + 55.0_f64) as u16
    }

}


pub struct EvalParams {
    pub mg_piece_value: [Eval; NUM_OF_PIECES],
    pub eg_piece_value: [Eval; NUM_OF_PIECES],
    pub psqt_eg: [[Eval; 64]; NUM_OF_PIECES],
    pub psqt_mg: [[Eval; 64]; NUM_OF_PIECES],
    pub mg_rook_mobility: [Eval; 15],
    pub eg_rook_mobility: [Eval; 15],
    pub mg_bishop_mobility: [Eval; 14],
    pub eg_bishop_mobility: [Eval; 14],
    pub mg_knight_mobility: [Eval; 9],
    pub eg_knight_mobility: [Eval; 9],
    pub mg_queen_mobility: [Eval; 28],
    pub eg_queen_mobility: [Eval; 28],
    pub mg_passed: [Eval; 8],
    pub eg_passed: [Eval; 8],
    pub open_king_file_punish_mg: Eval,
    pub mg_isolated_penalty: Eval,
    pub eg_isolated_penalty: Eval,
    pub mg_doubled_isolated_penalty: Eval,
    pub eg_doubled_isolated_penalty: Eval,
    pub mg_doubled_penalty: Eval,
    pub eg_doubled_penalty: Eval,
}

pub static mut EVAL_PARAMS: EvalParams = EvalParams {
    mg_piece_value: [88, 579, 404, 414, 1182, 0],
    eg_piece_value: [142, 682, 405, 389, 1182, 0],
    psqt_mg: [
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
                -50,-50,-50,-50,-50,-50,-50,-50,
                -50,-50,-50,-50,-50,-50,-50,-50,
                -50,-50,-50,-50,-50,-50,-50,-50,
                -50,-50,-50,-50,-50,-50,-50,-50,
                -50,-50,-50,-50,-50,-50,-50,-50,
                -50,-50,-50,-50,-50,-50,-50,-50,
            ],
        ],
    psqt_eg: [
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
    ],
    mg_rook_mobility: [-60,-20,2,3,3,11,22,31,40,40,41,48,57,57,62],
    eg_rook_mobility: [-78,-17,23,39,70,99,103,121,134,139,158,164,168,169,172],
    mg_bishop_mobility: [-48,-20,16,26,38,51,55,63,63,68,81,81,91,98],
    eg_bishop_mobility: [-59,-23,-3,13,24,42,54,57,65,73,78,86,88,97],
    mg_knight_mobility: [-62,-53,-12,-4,3,13,22,28,33],
    eg_knight_mobility: [-81,-56,-31,-16,5,11,17,20,25],
    mg_queen_mobility: [-30,-12,-8,-9,20,23,23,35,38,53,64,65,65,66,67,67,72,72,77,79,93,108,108,108,110,114,114,116],
    eg_queen_mobility: [-48,-30,-7,19,40,55,59,75,78,96,96,100,121,127,131,133,136,141,147,150,151,168,168,171,182,182,192,219],
    open_king_file_punish_mg: 50,
    mg_passed: [0, 10,17,15,62,168,276, 0],
    eg_passed: [0, 28,33,41,72,177,260, 0],
    mg_isolated_penalty: 23,
    eg_isolated_penalty: 12,
    mg_doubled_isolated_penalty: -18,
    eg_doubled_isolated_penalty: 37,
    mg_doubled_penalty: 28,
    eg_doubled_penalty: 36,
};

pub fn relevant_eval_params() -> Vec<*mut Eval> {
    let mut params = vec![];
    unsafe {
        params.push(std::ptr::addr_of_mut!(EVAL_PARAMS.mg_isolated_penalty));
        params.push(std::ptr::addr_of_mut!(EVAL_PARAMS.eg_isolated_penalty));
        params.push(std::ptr::addr_of_mut!(EVAL_PARAMS.mg_doubled_isolated_penalty));
        params.push(std::ptr::addr_of_mut!(EVAL_PARAMS.eg_doubled_isolated_penalty));
        params.push(std::ptr::addr_of_mut!(EVAL_PARAMS.mg_doubled_penalty));
        params.push(std::ptr::addr_of_mut!(EVAL_PARAMS.eg_doubled_penalty));
        for n in 0..EVAL_PARAMS.mg_piece_value.len() - 1 {
            params.push(std::ptr::addr_of_mut!(EVAL_PARAMS.mg_piece_value[n]));
            params.push(std::ptr::addr_of_mut!(EVAL_PARAMS.eg_piece_value[n]));
        }
        for n in 1..EVAL_PARAMS.mg_passed.len() - 1 {
            params.push(std::ptr::addr_of_mut!(EVAL_PARAMS.mg_passed[n]));
            params.push(std::ptr::addr_of_mut!(EVAL_PARAMS.eg_passed[n]));
        }
    }
    params
}

pub fn print_relevant_params() {
    print!("MG_VALUE:");
    unsafe {
        for value in EVAL_PARAMS.mg_piece_value {
            print!(" {}", value);
        }
        println!();
        print!("EG_VALUE:");
        for value in EVAL_PARAMS.eg_piece_value {
            print!(" {}", value);
        }
        println!();
        print!("MG_PASSED:");
        for value in EVAL_PARAMS.mg_passed {
            print!(" {}", value);
        }
        println!();
        print!("EG_PASSED:");
        for value in EVAL_PARAMS.eg_passed {
            print!(" {}", value);
        }
        println!();
        println!("ISOLATED (MG, EG): {} {}", EVAL_PARAMS.mg_isolated_penalty, EVAL_PARAMS.eg_isolated_penalty);
        println!("DOUBLED ISOLATED (MG, EG): {} {}", EVAL_PARAMS.mg_doubled_isolated_penalty, EVAL_PARAMS.eg_doubled_isolated_penalty);
        println!("DOUBLED (MG, EG): {} {}", EVAL_PARAMS.mg_doubled_penalty, EVAL_PARAMS.eg_doubled_penalty);
    }
}