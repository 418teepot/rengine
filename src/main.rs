#![feature(sync_unsafe_cell)]

use std::cell::SyncUnsafeCell;
use std::env;
use std::time::Duration;

use book::OPENING_BOOK;
use gamestate::GameState;
use movegen::RAY_FROM_TO;
use texel::texel_game;
use crate::texel::generate_texel_sample;
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
mod book;
mod texel;
mod lockless;
mod smpsearch;

fn initialize_lazy() {
    lazy_static::initialize(&RAY_FROM_TO);
    lazy_static::initialize(&KING_MOVES);
    lazy_static::initialize(&BISHOP_MAGICS_AND_PLAYS);
    lazy_static::initialize(&ROOK_MAGICS_AND_PLAYS);
    lazy_static::initialize(&KNIGHT_MOVES);
    lazy_static::initialize(&OPENING_BOOK);
}

fn main() {
     
    env::set_var("RUST_BACKTRACE", "1");
    initialize_lazy();
    uci_loop();
    
    
    /* 
    let texel_record = generate_texel_sample(64_000, Duration::from_millis(60));
    println!("{}", texel_record);
    */
    /* 
    let gs = GameState::new_from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1");
    let static_eval = gs.static_eval();
    println!("{}", static_eval);
    */
}
