use std::time::Duration;
use std::env;

use gamestate::GameState;
use movegen::RAY_FROM_TO;
use crate::search::{eval_into_white_viewpoint, iterative_deepening_timed};
use crate::{magic::{BISHOP_MAGICS_AND_PLAYS, ROOK_MAGICS_AND_PLAYS}, movegen::{KING_MOVES, KNIGHT_MOVES}};

#[macro_use]
extern crate lazy_static;
mod bitboard;
mod r#move;
mod gamestate;
mod zobrist;
mod magic;
mod movegen;
mod uci;
mod search;
mod eval;
mod tt;

fn initialize_lazy() {
    lazy_static::initialize(&RAY_FROM_TO);
    lazy_static::initialize(&KING_MOVES);
    lazy_static::initialize(&BISHOP_MAGICS_AND_PLAYS);
    lazy_static::initialize(&ROOK_MAGICS_AND_PLAYS);
    lazy_static::initialize(&KNIGHT_MOVES);
}

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    println!("Initializing lookup tables...");
    initialize_lazy();
    println!("Done!");
    // let mut gs = GameState::new_from_fen("r1bqkb1r/pppp1ppp/2n2n2/8/3pP3/2N2N2/PPP2PPP/R1BQKB1R w KQkq - 0 5");
    let mut gs = GameState::new_starting_pos();
    loop {
    let (best_move, best_eval) = iterative_deepening_timed(&mut gs, Duration::from_secs(30));
    println!("{}, {}", best_move.to_algebraic(), eval_into_white_viewpoint(best_eval, gs.side_to_move()));
    gs.apply_legal_move(best_move);
    }
}
