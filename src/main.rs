use std::process::exit;
use std::env;

use bitboard::Bitboard;
use gamestate::{GameState, WHITE, KING, BLACK, PAWN, BISHOP};
use magic::{relevant_slider_blockers, slider_plays_for_blockers};
use movegen::{bishop_move_bitboard, rook_move_bitboard, RAY_FROM_TO};
use r#move::Move;
use uci::perft_debug;

use crate::{magic::{BISHOP_MAGICS_AND_PLAYS, ROOK_MAGICS_AND_PLAYS}, uci::{perft, perft_timed, check_wrong_undo}, movegen::{KING_MOVES, KNIGHT_MOVES}};

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
    println!("Initializing lookup tables...");
    lazy_static::initialize(&RAY_FROM_TO);
    lazy_static::initialize(&KING_MOVES);
    lazy_static::initialize(&BISHOP_MAGICS_AND_PLAYS);
    lazy_static::initialize(&ROOK_MAGICS_AND_PLAYS);
    lazy_static::initialize(&KNIGHT_MOVES);
    println!("Done!");
    let mut gs = GameState::new_from_fen("r3k2r/p2pqpb1/1n2pnp1/1PpPN3/1p2P3/2N2Q1p/1PPBBPPP/R3K2R w KQkq 42 0 3");
    // gs.generate_moves();
    // gs.print_debug();
    // check_wrong_undo(&mut gs);
    // println!("{}", perft(&mut gs, 2));
    perft_debug(&mut gs, 1);    
}
