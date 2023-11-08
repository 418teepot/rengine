use std::collections::HashMap;
use std::io::Write;
use std::io::stdout;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use crate::gamestate::GameState;
use crate::search::SearchInfo;
use crate::search::iterative_deepening;
use crate::tt::TranspositionTable;
use crate::r#move::Move;
use std::io::{stdin};

const ENGINE_NAME: &'static str = "engine";

pub fn uci_loop() {
    let mut gamestate = GameState::new_starting_pos();
    loop {
        let mut input = String::new();
        stdin().read_line(&mut input).expect("Couldn't read string");
        input.trim().to_string();
        trim_newline(&mut input);
        let parts: Vec<&str> = input.split(' ').collect();
        let cmd = parts[0];
        match cmd {
            "uci" => cmd_uci(&parts[1..]),
            "isready" => cmd_isready(&parts[1..]),
            "position" => {
                gamestate = cmd_position(&parts[1..]);
            },
            "go" => cmd_go(&parts[1..], gamestate.clone()),
            "quit" => return,
            _ => println!("{}", cmd),
        }
    }
}

pub fn cmd_go(parts: &[&str], gamestate: GameState) {
    let mut part_index = 0;
    let mut settings: HashMap<String, i64> = HashMap::new();
    let mut is_infinite = false;
    while part_index < parts.len() {
        if parts[part_index] == "infinite" {
            is_infinite = true;
            part_index += 1;
            continue;
        }
        settings.insert(parts[part_index].to_string(), parts[part_index + 1].parse().unwrap());
        part_index += 2;
    }
    let stop_flag = Arc::new(Mutex::new(false));
    let mut gamestate = gamestate;
    if is_infinite {
        let stop_flag_clone = Arc::clone(&stop_flag);
        thread::spawn(move || {
            iterative_deepening(&mut gamestate, Duration::from_secs(86400), &mut SearchInfo::new(Instant::now()), &stop_flag_clone)
        });
        wait_for_stop();
        let mut stop_flag = stop_flag.lock().unwrap();
        *stop_flag = true;
        return;
    }
    if let Some(movetime) = settings.get("movetime") {
        iterative_deepening(&mut gamestate, Duration::from_millis(*movetime as u64), &mut SearchInfo::new(Instant::now()), &stop_flag);
        return;
    }
    let wtime = *settings.get("wtime").unwrap() as u64;
    let btime = *settings.get("btime").unwrap() as u64;
    let move_time = gamestate.calculate_movetime(wtime, btime);
    iterative_deepening(&mut gamestate, Duration::from_millis(move_time), &mut SearchInfo::new(Instant::now()), &stop_flag);
    
}

fn wait_for_stop() {
    loop {
        let mut input = String::new();
        stdin().read_line(&mut input).expect("Couldn't read string");
        input.trim().to_string();
        trim_newline(&mut input);
        if input == "stop" {
            return;
        }
    }
}

pub fn cmd_uci(_parts: &[&str]) {
    println!("id {}", ENGINE_NAME);
    println!("uciok");
}

pub fn cmd_isready(_parts: &[&str]) {
    println!("readyok");
}

pub fn cmd_position(parts: &[&str]) -> GameState {
    let mut rest;
    let mut gamestate = match parts[0] {
        "startpos" => {
            rest = &parts[1..];
            GameState::new_starting_pos()
        },
        "fen" => {
            rest = &parts[7..];
            GameState::new_from_fen(&parts[1..=6].join(" "))
        },
        _ => unreachable!(),
    };
    if rest.is_empty() {
        return gamestate;
    }
    match rest[0] {
        "moves" => {
            rest = &rest[1..];
        },
        _ => unreachable!(),
    };
    for &r#move in rest {
        let real_move = Move::from_text_move(&gamestate, r#move);
        gamestate.apply_legal_move(real_move);
    }
    gamestate
}

fn trim_newline(s: &mut String) {
    while s.ends_with('\n') || s.ends_with('\r') {
        s.pop();
    }
}

pub fn algebraic_to_index(algebraic: &str) -> Option<usize> {
    if algebraic.len() != 2 {
        return None; // Invalid input
    }
    
    let chars: Vec<char> = algebraic.chars().collect();
    let file = chars[0] as u8 - b'a';
    let rank = chars[1] as u8 - b'1';

    if file < 8 && rank < 8 {
        Some((rank * 8 + file) as usize)
    } else {
        None // Invalid algebraic notation
    }
}

pub fn extract_pv(state: &mut GameState, t_table: &TranspositionTable) -> String {
    let mut pv_string = String::new();
    
    let mut copy_state = state.clone();

    let mut depth = 0;
    while let Some(entry) = t_table.probe(copy_state.zobrist) {
        if depth > entry.depth {
            break;
        }
        let pvmove = entry.best_move;
        pv_string.push_str(&format!("{} ", pvmove.to_algebraic()));
        copy_state.apply_legal_move(pvmove);
        depth += 1;
    }

    pv_string
}

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