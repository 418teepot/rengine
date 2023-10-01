use std::process::exit;

use crate::magic::BISHOP_MAGICS_AND_PLAYS;

#[macro_use]
extern crate lazy_static;
mod bitboard;
mod r#move;
mod gamestate;
mod zobrist;
mod magic;
mod movegen;

fn main() {
    exit(0)
}
