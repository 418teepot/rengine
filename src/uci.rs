use std::io::Write;
use std::io::stdout;
use std::time::Instant;

use crate::gamestate::GameState;

pub fn perft_debug(state: &mut GameState, depth: u32) {
    assert!(depth > 0);
    let mut total = 0;
    for r#move in state.generate_legal_moves() {
        print!("{}: ", r#move.to_algebraic());
        stdout().flush().expect("Couldn't flush :(");
        state.apply_legal_move(r#move);
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

    for r#move in state.generate_legal_moves() {
        state.apply_legal_move(r#move);
        nodes += perft(state, depth - 1);
        state.undo_move();
    }
    nodes
}

pub fn check_wrong_undo(state: &mut GameState) {
    for r#move in state.generate_legal_moves() {
        let state_old = state.clone();
        state.apply_legal_move(r#move);
        state.undo_move();
        if state_old != *state {
            println!("Error found here");
            println!("{}", r#move.to_algebraic());
            state.print_debug();
        }
    }
}

pub fn perft_timed(state: &mut GameState, depth: u32) {
    let start = Instant::now();
    let result = perft(state, depth);
    let duration = start.elapsed().as_millis();
    let nps = (result as u128 / duration) * 1000;
    println!("Searched {result} nodes\n{nps} Np/s");
}