#![feature(sync_unsafe_cell)]
#![feature(adt_const_params)]
#![feature(let_chains)]
#![feature(lazy_cell)]
#![feature(float_next_up_down)]

use std::cell::SyncUnsafeCell;
use std::env;
use std::time::Duration;

use book::OPENING_BOOK;
use gamestate::{GameState, ROOK};
use lockless::{LockLessTransTable, LockLessValue};
use movegen::RAY_FROM_TO;
use r#move::Move;
use smpsearch::INFINITY;
use texel::{read_texel_sample_file, find_smallest_k, mean_square_error};
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

fn initialize_lazy() {
    lazy_static::initialize(&RAY_FROM_TO);
    lazy_static::initialize(&KING_MOVES);
    lazy_static::initialize(&BISHOP_MAGICS_AND_PLAYS);
    lazy_static::initialize(&ROOK_MAGICS_AND_PLAYS);
    lazy_static::initialize(&KNIGHT_MOVES);
    lazy_static::initialize(&OPENING_BOOK);
}

fn main() {
     
    let all_fens = read_texel_sample_file();
    let best_k = find_smallest_k(&all_fens);
    
    /* 
    env::set_var("RUST_BACKTRACE", "full");
    initialize_lazy();
    uci_loop();
    */
    
    
     
    // let texel_record = generate_texel_sample_threaded(64000, Duration::from_millis(60), 5);
    

    /*  
    let gs = GameState::new_from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1");
    let static_eval = gs.static_eval();
    let mut trans_table = LockLessTransTable::new(0);
    trans_table.insert(gs.zobrist, LockLessValue::new(Move::new_from_to(0, 1, ROOK), lockless::LockLessFlag::Exact, -INFINITY, 10));
    let entry = trans_table.get(gs.zobrist).unwrap();
    println!("{} {} {}", entry.best_move().to_algebraic(), entry.depth(), entry.value());
    */
}
