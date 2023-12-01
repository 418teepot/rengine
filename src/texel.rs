use std::{time::{Duration, Instant}, sync::{Arc, Mutex}, fs::File, io::Write, arch::x86_64::_MM_FROUND_NINT, cell::{UnsafeCell, SyncUnsafeCell}};

use rand::{distributions::WeightedIndex, thread_rng, prelude::*};

use crate::{gamestate::GameState, search::{SearchInfo, iterative_deepening, INFINITY, eval_into_white_viewpoint, Eval}, book::OPENING_BOOK, r#move::Move};

pub fn generate_texel_sample(samples: u32, movetime: Duration) -> String {
    let mut texel_samples = String::new();
    let mut file = File::create("resources/texel.txt").unwrap();
    for sample in 0..samples {
        println!("Running Texel Game {}/{}", sample+1, samples);
        let final_record = texel_game(movetime);
        file.write_all(&final_record.as_bytes()).unwrap();
        texel_samples.push_str(&final_record);
    }
    texel_samples
}

pub fn texel_game(movetime: Duration) -> String {
    let mut fen_record: String = String::new();
    let mut gamestate = GameState::new_starting_pos();
    let mut eval: Eval = 0;
    while !gamestate.is_game_over() {
        let mut search_info = SearchInfo::new(Instant::now());
        let stop_flag = Arc::new(Mutex::new(false));
        if let Some(entry) = OPENING_BOOK.get(&gamestate.to_reduced_book_fen()) {
            let moves: Vec<String> = entry.iter().map(|item| item.0.to_string()).collect();
            let weights: Vec<u32> = entry.iter().map(|item| item.1).collect(); 
            let dist = WeightedIndex::new(weights).unwrap();
            let mut rng = thread_rng();
            let r#move = moves[dist.sample(&mut rng)].to_string();
            let r#move = Move::from_text_move(&gamestate, &r#move);
            gamestate.apply_legal_move(r#move);
            continue;
        }
        fen_record.push_str(&gamestate.to_reduced_book_fen());
        fen_record.push('\n');
        let result_tuple = iterative_deepening::<false>(&mut gamestate, movetime, &mut search_info, &Arc::new(SyncUnsafeCell::new(false)));
        let r#move = result_tuple.0;
        eval = result_tuple.1;
        gamestate.apply_legal_move(r#move);
        if eval.abs() > 1200 {
            break;
        }
    }
    let winner: f32 = if gamestate.fifty_move_rule >= 100 || gamestate.has_repitition() {
        0.5
    } else if eval.abs() > 1200 {
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