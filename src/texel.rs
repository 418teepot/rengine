use std::{time::{Duration, Instant}, sync::{Arc, Mutex}, fs::File, io::{Write, BufReader, BufRead}, arch::x86_64::_MM_FROUND_NINT, cell::{UnsafeCell, SyncUnsafeCell}, thread};

use rand::{distributions::WeightedIndex, thread_rng, prelude::*};

use std::fs::OpenOptions;

use crate::{gamestate::{GameState, BLACK, Side, WHITE}, book::OPENING_BOOK, r#move::Move, smpsearch::{Eval, iterative_deepening, SearchProtocol, search, quiescent_search, AB_BOUND, SearchInfo, INFINITY, UciMode, ISMATE}, lockless::LockLessTransTable, eval::print_relevant_params};

pub fn generate_texel_sample_threaded(samples: u32, movetime: Duration, simul_threads: u8) -> String {
    let mut texel_samples = String::new();
    let mut file = OpenOptions::new().append(true).open("resources/texel.dat").expect("Can't find texel file");

    for sample in 0..(samples / simul_threads as u32) {
        let mut thread_pool = Vec::new();

        for i in 0..simul_threads {
            println!("Running Texel Game {}/{}", sample * simul_threads as u32 + i as u32 + 1, samples);
            thread_pool.push(thread::spawn(move || {
                texel_game(1000, 80)
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
        let final_record = texel_game(1000, 80);
        file.write_all(final_record.as_bytes()).unwrap();
        texel_samples.push_str(&final_record);
    }
    texel_samples
}

pub fn texel_game(total_time: u64, increment: u64) -> String {
    let mut fen_record: String = String::new();
    let mut gamestate = GameState::new_starting_pos();
    let mut eval: Eval = 0;
    let mut wtime = total_time;
    let mut btime = total_time;
    let trans_table = Arc::new(SyncUnsafeCell::new(LockLessTransTable::new()));
    while !gamestate.is_game_over() && wtime > 0 && btime > 0 {
        let stop_flag = Arc::new(SyncUnsafeCell::new(false));

        if eval.abs() <= 700 {
            fen_record.push_str(&gamestate.to_reduced_book_fen());
            fen_record.push('\n');
        }

        let movetime = Duration::from_millis(gamestate.calculate_movetime(wtime, btime, increment, increment));

        let timer = Instant::now();
        let result = search::<{ SearchProtocol::Texel }>(1, movetime, gamestate.clone(), Arc::clone(&stop_flag), 20, Arc::clone(&trans_table));
        let elapsed = timer.elapsed();

        if gamestate.side_to_move() == WHITE {
            wtime -= elapsed.as_millis() as u64;
            wtime += increment;
        } else {
            btime -= elapsed.as_millis() as u64;
            btime += increment;
        }

        eval = result.1;
        
        gamestate.apply_legal_move(result.0);
        
    }
    let winner: f32 = if gamestate.fifty_move_rule >= 100 || gamestate.has_repitition() {
        0.5
    } else if gamestate.is_game_over() {
        if gamestate.is_in_check() {
            if gamestate.side_to_move() == WHITE {
                0.0
            } else {
                1.0
            }
        } else {
            0.5
        }
    } else if btime == 0 {
        panic!("This shouldn't ever happen");
        1.0
    } else if wtime == 0 {
        panic!("This shouldn't ever happen");
        0.0
    } else {
        panic!("This shouldnt ever happen");
        0.5
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

pub fn eval_into_white_viewpoint(value: Eval, side_to_move: Side) -> Eval {
    if side_to_move == BLACK {
        -value
    } else {
        value
    }
}

pub fn read_texel_sample_file() -> Vec<(String, f64)> {
    let mut vec = vec![];
    let file = File::open("resources/texel.dat").unwrap();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line.unwrap();
        let mut splitted = line.rsplitn(2, ' ');
        let result: f64 = splitted.next().unwrap().parse().unwrap();
        let fen: String = splitted.next().unwrap().parse().unwrap();
        vec.push((fen, result));
    }
    vec
}

pub fn mean_square_error(k: f64, fen_and_values: &Vec<(String, f64)>) -> f64 {
    let mut error = 0.0;
    for (n, (fen, value)) in fen_and_values.iter().enumerate() {
        let mut gamestate = GameState::new_from_fen(&format!("{fen} 0 1"));

        let eval = {
            let tmp_eval = quiescent_search::<{SearchProtocol::Uci(UciMode::Infinite)}>(&mut gamestate, -INFINITY, INFINITY, 10, &mut SearchInfo::new(Duration::from_millis(0), Arc::new(SyncUnsafeCell::new(false))));
            // let tmp_eval = gamestate.static_eval();
            (if gamestate.side_to_move() == BLACK {
                -tmp_eval
            } else {
                tmp_eval
            }) as f64
        };

        let sigmoid = 1_f64 / (1_f64 + 10_f64.powf((-k * eval) / 400_f64));

        error += (value - sigmoid).powi(2);
    }
    error / fen_and_values.len() as f64
}

pub fn optimize_params(params: Vec<*mut Eval>) {
    let fen_and_values = read_texel_sample_file();
    let mut best_e = mean_square_error(K, &fen_and_values);
    let mut improved = true;
    while improved {
        
        improved = false;
        for &param in params.iter() {
            print_relevant_params();
            println!("Error: {best_e}\n");
            unsafe {
                *param += 1;
            }
            let new_e = mean_square_error(K, &fen_and_values);
            if new_e < best_e {
                best_e = new_e;
                improved = true;
            } else {
                unsafe {
                    *param -= 2;
                    let new_e = mean_square_error(K, &fen_and_values);
                    if new_e < best_e {
                        best_e = new_e;
                        improved = true;
                    } else {
                        *param += 1;
                    }
                }
            }
        }
    }
}

const DELTA_K: f64 = 0.0001_f64;
pub const K: f64 = 0.598_f64;
// const RIGHT_MAX: f64 = 15.00_f64;
// const LEFT_MAX: f64 = 0.00_f64;
pub fn find_smallest_k(fen_and_values: &Vec<(String, f64)>) -> f64 {
    // let mut left_max = LEFT_MAX;
    // let mut right_max = RIGHT_MAX;
    let mut best_k = 0.598_f64;
    let mut improved = true;
    let mut best_e = mean_square_error(best_k, fen_and_values);
    while improved {
        improved = false;
        println!("Error: {best_e} \t k: {best_k}");
        let new_k = best_k + DELTA_K;
        let new_e = mean_square_error(new_k, fen_and_values);
        if new_e < best_e {
            // left_max = best_k;
            best_e = new_e;
            best_k = new_k;
            improved = true;
        } else {
            let new_k = best_k - DELTA_K;
            let new_e = mean_square_error(new_k, fen_and_values);
            if new_e < best_e {
                // right_max = best_k;
                best_e = new_e;
                best_k = new_k;
                improved = true;
            }
        }
    }
    best_k
}