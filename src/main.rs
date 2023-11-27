use std::env;

use book::OPENING_BOOK;
use movegen::RAY_FROM_TO;
use crate::gamestate::{GameState, WHITE};
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
}
