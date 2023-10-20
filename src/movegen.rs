use crate::bitboard::{Bitboard, Square};
use crate::gamestate::{GameState, KING, Side, Piece, PAWN, ROOK, KNIGHT, BISHOP, QUEEN, NUM_OF_PIECES, WHITE, WHITE_QUEENSIDE_CASTLE, WHITE_KINGSIDE_CASTLE, BLACK_QUEENSIDE_CASTLE, NUM_OF_PLAYERS, BLACK_KINGSIDE_CASTLE, E1, C1, G1, E8, C8, G8};
use crate::magic::{mailbox64, mailbox, BISHOP_MAGICS_AND_PLAYS, magic_index, ROOK_MAGICS_AND_PLAYS};
use crate::r#move::{MoveList, self};
use crate::r#move::Move;

const FILE_A: usize = 0;
const FILE_H: usize = 7;
const RANK_2: usize = 1;
const RANK_8: usize = 7;
const RANK_1: usize = 0;
const RANK_7: usize = 6;

const CASTLE_WHITE_QUEENSIDE_FREE: Bitboard = Bitboard(0b00001110);
const CASTLE_WHITE_QUEENSIDE_CHECK_FREE: Bitboard = Bitboard(0b00011100);
const CASTLE_WHITE_KINGSIDE_FREE: Bitboard = Bitboard(0b01100000);
const CASTLE_WHITE_KINGSIDE_CHECK_FREE: Bitboard = Bitboard(0b01110000);

const CASTLE_BLACK_QUEENSIDE_FREE: Bitboard       = Bitboard(0b111_0000000000_0000000000_0000000000_0000000000_0000000000_0000000);
const CASTLE_BLACK_QUEENSIDE_CHECK_FREE: Bitboard = Bitboard(0b111_0000000000_0000000000_0000000000_0000000000_0000000000_00000000);
const CASTLE_BLACK_KINGSIDE_FREE: Bitboard        = Bitboard(0b11_0000000000_0000000000_0000000000_0000000000_0000000000_0000000000_0);
const CASTLE_BLACK_KINGSIDE_CHECK_FREE: Bitboard  = Bitboard(0b111_0000000000_0000000000_0000000000_0000000000_0000000000_0000000000);

impl GameState {
    pub fn generate_moves(&mut self) -> MoveList {
        let mut moves = MoveList::new();

        let our_side = self.side_to_move();
        let enemy_side = our_side ^ 1;
        let our_occupancy = self.occupancy(our_side);
        let enemy_occupancy = self.occupancy(enemy_side);
        let blockers = our_occupancy | enemy_occupancy;
        let king_danger_squares = self.king_danger_squares(our_side, blockers);

        let our_king_position = self.piece_boards[our_side][KING].next_piece_index();
        let king_moves = KING_MOVES[our_king_position] & !king_danger_squares & !our_occupancy;

        // King Moves
        for to_square in king_moves {
            if (Bitboard::square(to_square) & enemy_occupancy).is_filled() {
                moves.add_move(Move::new_capture(our_king_position, to_square, KING, self.find_piece_on(to_square, enemy_side)));
            }
            else {
                moves.add_move(Move::new_from_to(our_king_position, to_square, KING));
            }
        }

        let checkers = self.attackers_on_square(our_king_position, enemy_side, blockers);
        let num_checkers = checkers.0.count_ones();

        let mut capture_mask = Bitboard::full();
        let mut push_mask = Bitboard::full();

        if num_checkers > 1 {
            return moves;
        }
        if num_checkers == 1 {
            let checker_pos = checkers.next_piece_index();
            capture_mask = checkers;
            if is_slider(self.find_piece_on(checker_pos, enemy_side)) {
                push_mask = RAY_FROM_TO[checker_pos][our_king_position];
            } else {
                push_mask = Bitboard(0);
            }
        }

        let evade_check_mask = capture_mask | push_mask;

        let pin_hv = self.get_hv_pinmask(our_king_position, blockers, enemy_side); 
        let pin_d12 = self.get_diagonal_pinmask(our_king_position, blockers, enemy_side);

        // Knight moves
        for from_square in self.piece_boards[our_side][KNIGHT] & !(pin_hv | pin_d12) {
            let all_moves_bitboard = knight_move_bitboard(from_square) & evade_check_mask & !our_occupancy;
            // Captures
            for to_square in all_moves_bitboard & enemy_occupancy {
                moves.add_move(Move::new_capture(from_square, to_square, KNIGHT, self.find_piece_on(to_square, enemy_side)));
            }
            // Quiets
            for to_square in all_moves_bitboard & !enemy_occupancy {
                moves.add_move(Move::new_from_to(from_square, to_square, KNIGHT));
            }
        }

        // Rook moves
        for from_square in self.piece_boards[our_side][ROOK] & !pin_d12 {
            let all_moves_bitboard = if pin_hv.has(from_square) {
                rook_move_bitboard(from_square, blockers) & evade_check_mask & pin_hv & !our_occupancy
            }
            else {
                rook_move_bitboard(from_square, blockers) & evade_check_mask & !our_occupancy
            };
            // Captures
            for to_square in all_moves_bitboard & enemy_occupancy {
                moves.add_move(Move::new_capture(from_square, to_square, ROOK, self.find_piece_on(to_square, enemy_side)));
            }
            // Quiets
            for to_square in all_moves_bitboard & !enemy_occupancy {
                moves.add_move(Move::new_from_to(from_square, to_square, ROOK));
            }
        }

        // Bishop moves
        for from_square in self.piece_boards[our_side][BISHOP] & !pin_hv {
            let all_moves_bitboard = if pin_d12.has(from_square) {
                bishop_move_bitboard(from_square, blockers) & evade_check_mask & pin_d12 & !our_occupancy
            } else {
                bishop_move_bitboard(from_square, blockers) & evade_check_mask & !our_occupancy
            };
            // Captures
            for to_square in all_moves_bitboard & enemy_occupancy {
                moves.add_move(Move::new_capture(from_square, to_square, BISHOP, self.find_piece_on(to_square, enemy_side)));
            }
            // Quiets
            for to_square in all_moves_bitboard & !enemy_occupancy {
                moves.add_move(Move::new_from_to(from_square, to_square, BISHOP));
            }
        }

        // Queen-Rook moves
        for from_square in self.piece_boards[our_side][QUEEN] & !pin_d12 {
            let all_moves_bitboard = if pin_hv.has(from_square) {
                rook_move_bitboard(from_square, blockers) & evade_check_mask & pin_hv & !our_occupancy
            } else {
                rook_move_bitboard(from_square, blockers) & evade_check_mask & !our_occupancy
            };
            // Captures
            for to_square in all_moves_bitboard & enemy_occupancy {
                moves.add_move(Move::new_capture(from_square, to_square, QUEEN, self.find_piece_on(to_square, enemy_side)))
            }
            // Quiets
            for to_square in all_moves_bitboard & !enemy_occupancy {
                moves.add_move(Move::new_from_to(from_square, to_square, QUEEN));
            }
        }

        // Queen-Bishop moves
        for from_square in self.piece_boards[our_side][QUEEN] & !pin_hv {
            let all_moves_bitboard = if pin_d12.has(from_square) {
                bishop_move_bitboard(from_square, blockers) & evade_check_mask & pin_d12 & !our_occupancy
            } else {
                bishop_move_bitboard(from_square, blockers) & evade_check_mask & !our_occupancy
            };
            // Captures
            for to_square in all_moves_bitboard & enemy_occupancy {
                moves.add_move(Move::new_capture(from_square, to_square, QUEEN, self.find_piece_on(to_square, enemy_side)));
            }
            // Quiets
            for to_square in all_moves_bitboard & !enemy_occupancy {
                moves.add_move(Move::new_from_to(from_square, to_square, QUEEN));
            }
        }
        
        if our_side == WHITE {
            // Single pawn push
            let pawn_single_moves = ((self.piece_boards[our_side][PAWN] & !pin_d12) << 8) & !blockers & evade_check_mask;
            // Promotions
            for to_square in pawn_single_moves & RANK_BITMASK[RANK_8] {
                for piece in ROOK..=QUEEN {
                    moves.add_move(Move::new_quiet_promotion(to_square - 8, to_square, piece));
                }
            }
            // Quiets
            for to_square in pawn_single_moves & !RANK_BITMASK[RANK_8] {
                moves.add_move(Move::new_from_to(to_square - 8, to_square, PAWN));
            }

            // Double pawn push
            let pawn_double_moves = ((((self.piece_boards[our_side][PAWN] & !pin_d12 & RANK_BITMASK[RANK_2]) << 8) & !blockers) << 8) & !blockers & evade_check_mask;
            for to_square in pawn_double_moves {
                moves.add_move(Move::new_double_pawn_push(to_square - 16, to_square));
            }

            // Pawn capture up left
            let pawn_up_left_capture = ((self.piece_boards[our_side][PAWN] & !FILE_BITMASK[FILE_A] & !pin_hv) << 7) & enemy_occupancy & evade_check_mask;
            // No promotions
            for to_square in pawn_up_left_capture & !RANK_BITMASK[RANK_8] {
                if pin_d12.has(to_square - 7) && !pin_d12.has(to_square) {
                    continue;
                }
                moves.add_move(Move::new_capture(to_square - 7, to_square, PAWN, self.find_piece_on(to_square, enemy_side)));
            }
            // Promotions
            for to_square in pawn_up_left_capture & RANK_BITMASK[RANK_8] {
                if pin_d12.has(to_square - 7) && !pin_d12.has(to_square) {
                    continue;
                }
                for piece in ROOK..=QUEEN {
                    moves.add_move(Move::new_capture_promotion(to_square - 7, to_square, piece, self.find_piece_on(to_square, enemy_side)));
                }
            }

            // Pawn capture up right
            let pawn_up_right_capture = ((self.piece_boards[our_side][PAWN] & !FILE_BITMASK[FILE_H] & !pin_hv) << 9) & enemy_occupancy & evade_check_mask;
            // No promotions
            for to_square in pawn_up_right_capture & !RANK_BITMASK[RANK_8] {
                if pin_d12.has(to_square - 9) && !pin_d12.has(to_square) {
                    continue;
                }
                moves.add_move(Move::new_capture(to_square - 9, to_square, PAWN, self.find_piece_on(to_square, enemy_side)));
            }
            // Promotions
            for to_square in pawn_up_right_capture & RANK_BITMASK[RANK_8] {
                if pin_d12.has(to_square - 9) && !pin_d12.has(to_square) {
                    continue;
                }
                for piece in ROOK..=QUEEN {
                    moves.add_move(Move::new_capture_promotion(to_square - 9, to_square, piece, self.find_piece_on(to_square, enemy_side)));
                }
            }

            // EnPassant
            if self.en_passant_board.is_filled() {
                let en_passant_pos = self.en_passant_board.next_piece_index();
                if (((self.piece_boards[our_side][PAWN] & !FILE_BITMASK[FILE_A]) << 7) & self.en_passant_board).is_filled() {
                    let r#move = Move::new_en_passant_capture(en_passant_pos - 7, en_passant_pos);
                    self.apply_move(r#move);
                    if self.attackers_on_square(our_king_position, enemy_side, self.occupancy(our_side) | self.occupancy(enemy_side)).0.count_ones() == 0 {
                        moves.add_move(r#move);
                    }
                    self.undo_move();
                }
                if (((self.piece_boards[our_side][PAWN] & !FILE_BITMASK[FILE_H]) << 9) & self.en_passant_board).is_filled() {
                    let r#move = Move::new_en_passant_capture(en_passant_pos - 9, en_passant_pos);
                    self.apply_move(r#move);
                    if self.attackers_on_square(our_king_position, enemy_side, self.occupancy(our_side) | self.occupancy(enemy_side)).0.count_ones() == 0 {
                        moves.add_move(r#move);
                    }
                    self.undo_move();
                }
            }

            // Castle
            if self.castling_rights[WHITE_QUEENSIDE_CASTLE] 
            && (blockers & CASTLE_WHITE_QUEENSIDE_FREE).is_empty() 
            && (king_danger_squares & CASTLE_WHITE_QUEENSIDE_CHECK_FREE).is_empty() {
                moves.add_move(Move::new_castle(r#move::CastlingSide::QueenSide, E1, C1));
            }
            if self.castling_rights[WHITE_KINGSIDE_CASTLE] 
            && (blockers & CASTLE_WHITE_KINGSIDE_FREE).is_empty() 
            && (king_danger_squares & CASTLE_WHITE_KINGSIDE_CHECK_FREE).is_empty() {
                moves.add_move(Move::new_castle(r#move::CastlingSide::KingSide, E1, G1));
            }
        }
        else {
            // Single pawn push
            let pawn_single_moves = ((self.piece_boards[our_side][PAWN] & !pin_d12) >> 8) & !blockers & evade_check_mask;
            // Promotions
            for to_square in pawn_single_moves & RANK_BITMASK[RANK_1] {
                for piece in ROOK..=QUEEN {
                    moves.add_move(Move::new_quiet_promotion(to_square + 8, to_square, piece));
                }
            }
            // Quiets
            for to_square in pawn_single_moves & !RANK_BITMASK[RANK_1] {
                moves.add_move(Move::new_from_to(to_square + 8, to_square, PAWN));
            }

            // Double pawn push
            let pawn_double_moves = ((((self.piece_boards[our_side][PAWN] & !pin_d12 & RANK_BITMASK[RANK_7]) >> 8) & !blockers) >> 8) & !blockers & evade_check_mask;
            for to_square in pawn_double_moves {
                moves.add_move(Move::new_double_pawn_push(to_square + 16, to_square));
            }

            // Pawn capture up left
            let pawn_down_right_capture = ((self.piece_boards[our_side][PAWN] & !pin_hv & !FILE_BITMASK[FILE_H]) >> 7) & enemy_occupancy & evade_check_mask;
            // No promotions
            for to_square in pawn_down_right_capture & !RANK_BITMASK[RANK_1] {
                if pin_d12.has(to_square + 7) && !pin_d12.has(to_square) {
                    continue;
                }
                moves.add_move(Move::new_capture(to_square + 7, to_square, PAWN, self.find_piece_on(to_square, enemy_side)));
            }
            // Promotions
            for to_square in pawn_down_right_capture & RANK_BITMASK[RANK_1] {
                if pin_d12.has(to_square + 7) && !pin_d12.has(to_square) {
                    continue;
                }
                for piece in ROOK..=QUEEN {
                    moves.add_move(Move::new_capture_promotion(to_square + 7, to_square, piece, self.find_piece_on(to_square, enemy_side)));
                }
            }

            // Pawn capture up right
            let pawn_down_left_capture = ((self.piece_boards[our_side][PAWN] & !pin_hv & !FILE_BITMASK[FILE_A]) >> 9) & enemy_occupancy & evade_check_mask;
            // No promotions
            for to_square in pawn_down_left_capture & !RANK_BITMASK[RANK_1] {
                if pin_d12.has(to_square + 9) && !pin_d12.has(to_square) {
                    continue;
                }
                moves.add_move(Move::new_capture(to_square + 9, to_square, PAWN, self.find_piece_on(to_square, enemy_side)));
            }
            // Promotions
            for to_square in pawn_down_left_capture & RANK_BITMASK[RANK_1] {
                if pin_d12.has(to_square + 9) && !pin_d12.has(to_square) {
                    continue;
                }
                for piece in ROOK..=QUEEN {
                    moves.add_move(Move::new_capture_promotion(to_square + 9, to_square, piece, self.find_piece_on(to_square, enemy_side)));
                }
            }

            // EnPassant
            if self.en_passant_board.is_filled() {
                let en_passant_pos = self.en_passant_board.next_piece_index();
                if (((self.piece_boards[our_side][PAWN] & !FILE_BITMASK[FILE_H]) >> 7) & self.en_passant_board).is_filled() {
                    let r#move = Move::new_en_passant_capture(en_passant_pos + 7, en_passant_pos);
                    self.apply_move(r#move);
                    if self.attackers_on_square(our_king_position, enemy_side, self.occupancy(our_side) | self.occupancy(enemy_side)).0.count_ones() == 0 {
                        moves.add_move(r#move);
                    }
                    self.undo_move();
                }
                if (((self.piece_boards[our_side][PAWN] & !FILE_BITMASK[FILE_A]) >> 9) & self.en_passant_board).is_filled() {
                    let r#move = Move::new_en_passant_capture(en_passant_pos + 9, en_passant_pos);
                    self.apply_move(r#move);
                    if self.attackers_on_square(our_king_position, enemy_side, self.occupancy(our_side) | self.occupancy(enemy_side)).0.count_ones() == 0 {
                        moves.add_move(r#move);
                    }
                    self.undo_move();
                }
            }

            // Castle
            if self.castling_rights[BLACK_QUEENSIDE_CASTLE] 
            && (blockers & CASTLE_BLACK_QUEENSIDE_FREE).is_empty() 
            && (king_danger_squares & CASTLE_BLACK_QUEENSIDE_CHECK_FREE).is_empty() {
                moves.add_move(Move::new_castle(r#move::CastlingSide::QueenSide, E8, C8));
            }
            if self.castling_rights[BLACK_KINGSIDE_CASTLE]
            && (blockers & CASTLE_BLACK_KINGSIDE_FREE).is_empty() 
            && (king_danger_squares & CASTLE_BLACK_KINGSIDE_CHECK_FREE).is_empty() {
                moves.add_move(Move::new_castle(r#move::CastlingSide::KingSide, E8, G8));
            }
        }

        moves
    }

    pub fn get_hv_pinmask(&self, king_square: Square, blockers: Bitboard, enemy_side: Side) -> Bitboard {
        let king_blockers = rook_move_bitboard(king_square, blockers);
        let without_king_blockers = blockers & !king_blockers;
        let moves_from_king = rook_move_bitboard(king_square, without_king_blockers);
        
        let mut maybe_moves_to_king = Bitboard(0);
        for square in self.piece_boards[enemy_side][ROOK] & moves_from_king {
            maybe_moves_to_king |= rook_move_bitboard(square, without_king_blockers) | Bitboard::square(square);
        }
        for square in self.piece_boards[enemy_side][QUEEN] & moves_from_king {
            maybe_moves_to_king |= rook_move_bitboard(square, without_king_blockers) | Bitboard::square(square);
        }

        maybe_moves_to_king & moves_from_king
    }

    pub fn get_diagonal_pinmask(&self, king_square: Square, blockers: Bitboard, enemy_side: Side) -> Bitboard {
        let king_blockers = bishop_move_bitboard(king_square, blockers);
        let without_king_blockers = blockers & !king_blockers;
        let moves_from_king = bishop_move_bitboard(king_square, without_king_blockers);

        let mut maybe_moves_to_king = Bitboard(0);
        for square in self.piece_boards[enemy_side][BISHOP] & moves_from_king {
            maybe_moves_to_king |= bishop_move_bitboard(square, without_king_blockers) | Bitboard::square(square);
        }
        for square in self.piece_boards[enemy_side][QUEEN] & moves_from_king {
            maybe_moves_to_king |= bishop_move_bitboard(square, without_king_blockers) | Bitboard::square(square);
        }

        maybe_moves_to_king & moves_from_king
    }

    fn attackers_on_square(&self, square: Square, enemy_side: Side, blockers: Bitboard) -> Bitboard {
        let square_bitboard = Bitboard::square(square);
        knight_move_bitboard(square) & self.piece_boards[enemy_side][KNIGHT]
        | rook_move_bitboard(square, blockers) & self.piece_boards[enemy_side][ROOK]
        | bishop_move_bitboard(square, blockers) & self.piece_boards[enemy_side][BISHOP]
        | queen_move_bitboard(square, blockers) & self.piece_boards[enemy_side][QUEEN]
        | if enemy_side == WHITE {
            (((square_bitboard & !FILE_BITMASK[FILE_H]) >> 7) | ((square_bitboard & !FILE_BITMASK[FILE_A]) >> 9)) & self.piece_boards[enemy_side][PAWN]
        } else {
            (((square_bitboard & !FILE_BITMASK[FILE_A]) << 7) | ((square_bitboard & !FILE_BITMASK[FILE_H]) << 9)) & self.piece_boards[enemy_side][PAWN]
        }
    }

    fn attacked_squares(&self, enemy_side: Side, blockers: Bitboard) -> Bitboard {
        let mut danger_squares = Bitboard::empty();

        danger_squares |= king_move_bitboard(self.piece_boards[enemy_side][KING].next_piece_index());

        for queen_index in self.piece_boards[enemy_side][QUEEN] {
            danger_squares |= queen_move_bitboard(queen_index, blockers);
        }

        for rook_index in self.piece_boards[enemy_side][ROOK] {
            danger_squares |= rook_move_bitboard(rook_index, blockers);
        }

        for bishop_index in self.piece_boards[enemy_side][BISHOP] {
            danger_squares |= bishop_move_bitboard(bishop_index, blockers);
        }

        for knight_index in self.piece_boards[enemy_side][KNIGHT] {
            danger_squares |= knight_move_bitboard(knight_index);
        }

        if enemy_side == WHITE {
            danger_squares |= (self.piece_boards[enemy_side][PAWN] & !FILE_BITMASK[FILE_A]) << 9;
            danger_squares |= (self.piece_boards[enemy_side][PAWN] & !FILE_BITMASK[FILE_H]) << 7;
        }
        else {
            danger_squares |= (self.piece_boards[enemy_side][PAWN] & !FILE_BITMASK[FILE_H]) >> 7;
            danger_squares |= (self.piece_boards[enemy_side][PAWN] & !FILE_BITMASK[FILE_A]) >> 9;
        }

        danger_squares
    }

    #[inline(always)]
    pub fn king_danger_squares(&self, our_side: Side, blockers: Bitboard) -> Bitboard {
        self.attacked_squares(our_side ^ 1, blockers & !self.piece_boards[our_side][KING])
    }

    pub fn occupancy(&self, side: Side) -> Bitboard {
        self.piece_boards[side][PAWN] | self.piece_boards[side][ROOK] | self.piece_boards[side][KNIGHT] | self.piece_boards[side][BISHOP] | self.piece_boards[side][QUEEN] | self.piece_boards[side][KING]
    }

    fn find_piece_on(&self, square: Square, side: Side) -> Piece {
        let square_mask = Bitboard::square(square);
        for piece in 0..NUM_OF_PIECES {
            if (self.piece_boards[side][piece] & square_mask).is_filled() {
                return piece;
            }
        }
        unreachable!("Searched for piece on empty square.")
    }

    pub fn find_piece_on_all(&self, square: Square) -> Option<(Side, Piece)> {
        for piece in 0..NUM_OF_PIECES {
            for side in 0..NUM_OF_PLAYERS {
                if self.piece_boards[side][piece].has(square) {
                    return Some((side, piece));
                }
            }
        }
        None
    }
}

#[inline(always)]
fn queen_move_bitboard(square: Square, blockers: Bitboard) -> Bitboard {
    bishop_move_bitboard(square, blockers) | rook_move_bitboard(square, blockers)
}

#[inline(always)]
pub fn bishop_move_bitboard(square: Square, blockers: Bitboard) -> Bitboard {
    let index = magic_index(BISHOP_MAGICS_AND_PLAYS[square].0, blockers);
    BISHOP_MAGICS_AND_PLAYS[square].1[index]
}

#[inline(always)]
pub fn rook_move_bitboard(square: Square, blockers: Bitboard) -> Bitboard {
    let index = magic_index(ROOK_MAGICS_AND_PLAYS[square].0, blockers);
    ROOK_MAGICS_AND_PLAYS[square].1[index]
}

#[inline(always)]
fn knight_move_bitboard(square: Square) -> Bitboard {
    KNIGHT_MOVES[square]
}

#[inline(always)]
fn king_move_bitboard(square: Square) -> Bitboard {
    KING_MOVES[square]
}

#[inline(always)]
fn is_slider(piece: Piece) -> bool {
    piece > PAWN && piece < KING
}




static KING_RAYS: [i8; 8] = [-11, -10, -9, -1, 1,  9, 10, 11];
static KNIGHT_RAYS: [i8; 8] = [-21, -19,-12, -8, 8, 12, 19, 21];

lazy_static! {
    pub static ref KING_MOVES: [Bitboard; 64] = {
        let mut king_moves = [Bitboard(0); 64];
        for square in 0..64 {
            let mut bitmask = Bitboard(0);
            for ray in KING_RAYS {
                if mailbox[(mailbox64[square] + ray) as usize] != -1 {
                    bitmask |= Bitboard::square(mailbox[(mailbox64[square] + ray) as usize] as usize);
                }
            }
            king_moves[square] = bitmask;
        }
        king_moves
    };

    pub static ref KNIGHT_MOVES: [Bitboard; 64] = {
        let mut knight_moves = [Bitboard(0); 64];
        for square in 0..64 {
            let mut bitmask = Bitboard(0);
            for ray in KNIGHT_RAYS {
                if mailbox[(mailbox64[square] + ray) as usize] != -1 {
                    bitmask |= Bitboard::square(mailbox[(mailbox64[square] + ray) as usize] as usize);
                }
            }
            knight_moves[square] = bitmask;
        }
        knight_moves
    };

    pub static ref FILE_BITMASK: [Bitboard; 8] = {
        let mut bitmasks = [Bitboard(0); 8];
        for file in 0..8 {
            let mut file_bitmask = Bitboard(0);
            for rank in 0..8 {
                let square = rank * 8 + file;
                file_bitmask |= Bitboard::square(square);
            }
            bitmasks[file] = file_bitmask;
        }
        bitmasks
    };

    pub static ref RANK_BITMASK: [Bitboard; 8] = {
        let mut bitmasks = [Bitboard(0); 8];
        for rank in 0..8 {
            let mut rank_bitmask = Bitboard(0);
            for file in 0..8 {
                let square = rank * 8 + file;
                rank_bitmask |= Bitboard::square(square);
            }
            bitmasks[rank] = rank_bitmask;
        }
        bitmasks
    };

    pub static ref RAY_FROM_TO: [[Bitboard; 64]; 64] = {
        /// My thanks to Ava for helping me out with her awesome piece of code <3
        const RAYS: [i8; 8] = [9, 10, 11, -1, -11, -10, -9, 1];
        fn r_from_to(from: usize, to: usize) -> u64 {
            for ray in RAYS {
                let mut bitmask = 0;
                let mut i = 0;
                loop {
                    let current_square = mailbox[(mailbox64[from] + ray * i) as usize];
                    if current_square == -1 {
                        break;
                    }
                    bitmask |= 1 << current_square;
                    if current_square == to.try_into().unwrap() {
                        return bitmask;
                    }
                    i += 1;
                }
            }
            0
        }
        let mut rays = [[Bitboard(0); 64]; 64];
        for from in 0..64 {
            for to in 0..64 {
                rays[from][to] = Bitboard(r_from_to(from, to));
            } 
        }
        rays
    };

}

#[cfg(test)]
mod tests {
    use crate::{gamestate::GameState, uci::perft};

    #[test]
    fn perft_starting_pos() {
        let mut starting_pos = GameState::new_starting_pos();
        assert_eq!(perft(&mut starting_pos, 0), 1);
        assert_eq!(perft(&mut starting_pos, 1), 20);
        assert_eq!(perft(&mut starting_pos, 2), 400);
        assert_eq!(perft(&mut starting_pos, 3), 8902);
        assert_eq!(perft(&mut starting_pos, 4), 197281);
        assert_eq!(perft(&mut starting_pos, 5), 4865609);
    }

    #[test]
    fn perft_kiwipete() {
        let mut starting_pos = GameState::new_from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
        assert_eq!(perft(&mut starting_pos, 1), 48);
        assert_eq!(perft(&mut starting_pos, 2), 2039);
        assert_eq!(perft(&mut starting_pos, 3), 97862);
        assert_eq!(perft(&mut starting_pos, 4), 4085603);
        assert_eq!(perft(&mut starting_pos, 5), 193690690);
    }
}