use std::time::{Duration, Instant};
use std::env;

use gamestate::GameState;
use movegen::RAY_FROM_TO;
use crate::search::{eval_into_white_viewpoint, iterative_deepening_timed, SearchData, SearchInfo, pick_best_move_timed};
use crate::uci::extract_pv;
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
    
    let mut gs = GameState::new_starting_pos();
    let mut search_info = SearchInfo::new(Instant::now(), Duration::from_secs(30));
    loop {
        if gs.game_is_over() {
            break;
        }
        let (best_move, best_eval) = iterative_deepening_timed(&mut gs, Duration::from_secs(120), &mut search_info);
        gs.apply_legal_move(best_move);
    }
    
    
}
