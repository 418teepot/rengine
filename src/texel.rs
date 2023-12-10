use std::{time::{Duration, Instant}, sync::{Arc, Mutex}, fs::File, io::Write, arch::x86_64::_MM_FROUND_NINT, cell::{UnsafeCell, SyncUnsafeCell}, thread};

use rand::{distributions::WeightedIndex, thread_rng, prelude::*};

use crate::{gamestate::{GameState, BLACK, Side}, book::OPENING_BOOK, r#move::Move, smpsearch::{Eval, iterative_deepening, SearchProtocol, search}, lockless::LockLessTransTable};

pub fn generate_texel_sample_threaded(samples: u32, movetime: Duration, simul_threads: u8) -> String {
    let mut texel_samples = String::new();
    let mut file = File::create("resources/texel.dat").unwrap();

    for sample in 0..(samples / simul_threads as u32) {
        let mut thread_pool = Vec::new();

        for i in 0..simul_threads {
            println!("Running Texel Game {}/{}", sample * simul_threads as u32 + i as u32 + 1, samples);
            thread_pool.push(thread::spawn(move || {
                texel_game(movetime)
            }));
        }

        for thread in thread_pool {
            let result = thread.join().unwrap_or("\n".to_string());
            file.write_all(result.as_bytes()).unwrap();
            // texel_samples.push_str(&result);
        }
    }

    texel_samples
}

pub fn generate_texel_sample(samples: u32, movetime: Duration) -> String {
    let mut texel_samples = String::new();
    let mut file = File::create("resources/texel.txt").unwrap();
    for sample in 0..samples {
        println!("Running Texel Game {}/{}", sample+1, samples);
        let final_record = texel_game(movetime);
        file.write_all(final_record.as_bytes()).unwrap();
        texel_samples.push_str(&final_record);
    }
    texel_samples
}

pub fn texel_game(movetime: Duration) -> String {
    let mut fen_record: String = String::new();
    let mut gamestate = GameState::new_starting_pos();
    let mut eval: Eval = 0;
    while !gamestate.is_game_over() {
        let stop_flag = Arc::new(SyncUnsafeCell::new(false));
        let trans_table = Arc::new(SyncUnsafeCell::new(LockLessTransTable::new()));
        fen_record.push_str(&gamestate.to_reduced_book_fen());
        fen_record.push('\n');
        let result = search::<{ SearchProtocol::Texel }>(1, movetime, gamestate.clone(), Arc::clone(&stop_flag), 20, trans_table);
        eval = result.1;
        gamestate.apply_legal_move(result.0);
        if eval.abs() > 600 {
            break;
        }
    }
    let winner: f32 = if gamestate.fifty_move_rule >= 100 || gamestate.has_repitition() {
        0.5
    } else if eval.abs() > 600 {
        let eval_normalised = eval_into_white_viewpoint(eval, gamestate.side_to_move());
        if eval_normalised < 0.0 {
            0.0
        } else {
            1.0
        }
    } else {
        0.0
    };
    let mut final_record = String::new();
    for line in fen_record.lines() {
        final_record.push_str(line);
        final_record.push(' ');
        final_record.push_str(&winner.to_string());
        final_record.push('\n');
    }
    final_record
}

pub fn eval_into_white_viewpoint(value: Eval, side_to_move: Side) -> f64 {
    if side_to_move == BLACK {
        -value as f64 / 100_f64
    } else {
        value as f64 / 100_f64
    }
} 
