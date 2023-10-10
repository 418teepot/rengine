use std::string;
use std::io::Write;
use std::io::stdout;

use crate::gamestate::GameState;

pub fn perft_debug(state: &mut GameState, depth: u32) {
    assert!(depth > 0);
    let mut total = 0;
    for r#move in state.generate_moves() {
        print!("{}: ", r#move.to_algebraic());
        stdout().flush().expect("Couldn't flush :(");
        state.apply_move(r#move);
        let perft_result = perft(state, depth - 1);
        total += perft_result;
        println!("{}", perft_result);
        state.undo_move();
    }
    println!("Perft({}): {}", depth, total);
}

pub fn perft(state: &mut GameState, depth: u32) -> u32 {
    if depth == 0 {
        return 1;
    }
    let mut nodes = 0;

    for r#move in state.generate_moves() {
        state.apply_move(r#move);
        nodes += perft(state, depth - 1);
        state.undo_move();
    }
    nodes
}