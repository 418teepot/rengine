use std::env;

use movegen::RAY_FROM_TO;
use crate::uci::uci_loop;
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
    uci_loop();
    /* 
    
    let mut search_info = SearchInfo::new(Instant::now());
    loop {
        if gs.game_is_over() {
            break;
        }
        let (best_move, best_eval) = iterative_deepening(&mut gs, Duration::from_secs(15), &mut search_info);
        gs.apply_legal_move(best_move);
    }
    */
    
    
}
