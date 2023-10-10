use std::process::exit;
use std::env;

use bitboard::Bitboard;
use gamestate::{GameState, WHITE, KING, BLACK, PAWN, BISHOP};
use magic::{relevant_slider_blockers, slider_plays_for_blockers};
use movegen::{bishop_move_bitboard, rook_move_bitboard};
use r#move::Move;
use uci::perft_debug;

use crate::{magic::BISHOP_MAGICS_AND_PLAYS, uci::perft};

#[macro_use]
extern crate lazy_static;
mod bitboard;
mod r#move;
mod gamestate;
mod zobrist;
mod magic;
mod movegen;
mod uci;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    let mut gs = GameState::new_starting_pos();
    perft_debug(&mut gs, 4);
}
