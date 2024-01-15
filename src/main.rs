#![feature(sync_unsafe_cell)]
#![feature(adt_const_params)]
#![feature(let_chains)]
#![feature(lazy_cell)]
#![feature(float_next_up_down)]

use std::cell::SyncUnsafeCell;
use std::env;
use std::time::Duration;

use book::OPENING_BOOK;
use eval::relevant_eval_params;
use gamestate::{GameState, ROOK};
use lockless::{LockLessTransTable, LockLessValue};
use movegen::RAY_FROM_TO;
use r#move::Move;
use smpsearch::INFINITY;
use texel::{read_texel_sample_file, find_smallest_k, mean_square_error, optimize_params};
use crate::texel::{generate_texel_sample, generate_texel_sample_threaded};
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
mod eval;
mod tt;
mod book;
mod texel;
mod lockless;
mod smpsearch;
mod book_data;
mod smac;

fn initialize_lazy() {
    lazy_static::initialize(&RAY_FROM_TO);
    lazy_static::initialize(&KING_MOVES);
    lazy_static::initialize(&BISHOP_MAGICS_AND_PLAYS);
    lazy_static::initialize(&ROOK_MAGICS_AND_PLAYS);
    lazy_static::initialize(&KNIGHT_MOVES);
    lazy_static::initialize(&OPENING_BOOK);
}

fn main() {
    /* 
    let all_fens = read_texel_sample_file();
    let best_k = find_smallest_k(&all_fens);
    */

    
    // let params = relevant_eval_params();
    // optimize_params(params);
    

    /*  
    env::set_var("RUST_BACKTRACE", "full");
    initialize_lazy();
    uci_loop();
    */
    
    // let texel_record = generate_texel_sample_threaded(64000, Duration::from_millis(60), 10);
    
     let _ = crate::smac::smac();
}
