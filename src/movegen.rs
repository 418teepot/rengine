use crate::bitboard::{Bitboard, Square};
use crate::gamestate::{GameState, KING, Side, Piece, PAWN, ROOK, KNIGHT, BISHOP, QUEEN, NUM_OF_PIECES, WHITE};
use crate::magic::{mailbox64, mailbox, BISHOP_MAGICS_AND_PLAYS, magic_index, ROOK_MAGICS_AND_PLAYS};
use crate::r#move::MoveList;
use crate::r#move::Move;

const FILE_A: usize = 0;
const FILE_H: usize = 7;

impl GameState {
    fn generate_moves(&mut self) -> MoveList {
        let mut moves = MoveList::new();

        let our_side = self.side_to_move();
        let enemy_side = our_side ^ 1;
        let our_occupancy = self.occupancy(our_side);
        let enemy_occupancy = self.occupancy(enemy_side);
        let blockers = our_occupancy | enemy_occupancy;

        let our_king_position = self.piece_boards[our_side][KING].next_piece_index();
        let king_moves = KING_MOVES[our_king_position] & !self.king_danger_squares(our_side, blockers) & !our_occupancy;

        for to_square in king_moves {
            if (Bitboard::square(to_square) & enemy_occupancy).is_filled() {
                moves.add_move(Move::new_capture(our_king_position, to_square, KING, self.find_piece_on(to_square, enemy_side)));
            }
            else {
                moves.add_move(Move::new_from_to(our_king_position, to_square, KING));
            }
        }

        moves
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
            danger_squares |= (self.piece_boards[enemy_side][PAWN] & FILE_BITMASK[FILE_A]) << 9;
            danger_squares |= (self.piece_boards[enemy_side][PAWN] & FILE_BITMASK[FILE_H]) << 7;
        }
        else {
            danger_squares |= (self.piece_boards[enemy_side][PAWN] & FILE_BITMASK[FILE_A]) >> 7;
            danger_squares |= (self.piece_boards[enemy_side][PAWN] & FILE_BITMASK[FILE_H]) >> 9;
        }

        danger_squares
    }

    fn king_danger_squares(&self, our_side: Side, blockers: Bitboard) -> Bitboard {
        self.attacked_squares(our_side ^ 1, blockers & !self.piece_boards[our_side][KING])
    }

    fn occupancy(&self, side: Side) -> Bitboard {
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
}

fn queen_move_bitboard(square: Square, blockers: Bitboard) -> Bitboard {
    bishop_move_bitboard(square, blockers) | rook_move_bitboard(square, blockers)
}

fn bishop_move_bitboard(square: Square, blockers: Bitboard) -> Bitboard {
    let index = magic_index(BISHOP_MAGICS_AND_PLAYS[square].0, blockers);
    BISHOP_MAGICS_AND_PLAYS[square].1[index]
}

fn rook_move_bitboard(square: Square, blockers: Bitboard) -> Bitboard {
    let index = magic_index(ROOK_MAGICS_AND_PLAYS[square].0, blockers);
    ROOK_MAGICS_AND_PLAYS[square].1[index]
}

fn knight_move_bitboard(square: Square) -> Bitboard {
    KNIGHT_MOVES[square]
}

fn king_move_bitboard(square: Square) -> Bitboard {
    KING_MOVES[square]
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
        return king_moves;
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
}