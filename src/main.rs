use std::process::exit;
use std::env;

use gamestate::{GameState, WHITE, KING, BLACK, PAWN};
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
    // gs.apply_move(Move::new_from_to(14, 22, PAWN));
    perft_debug(&mut gs, 4);
}
