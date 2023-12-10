use std::{cell::SyncUnsafeCell, sync::Arc, time::{Duration, Instant}, marker::ConstParamTy, thread};

use rand::{distributions::WeightedIndex, thread_rng};

use crate::{r#move::{Move, MoveList}, bitboard::NUM_OF_SQUARES, gamestate::{GameState, NUM_OF_PLAYERS, Side, NUM_OF_PIECES}, lockless::{LockLessTransTable, LockLessValue, LockLessFlag}, book::OPENING_BOOK};

use rand::prelude::*;

const MAX_SEARCH_DEPTH: u8 = 30;
const MAX_KILLER_MOVES: usize = 2;
const MAX_QUIESCENT_DEPTH: u8 = 10;
const MAX_CHECK_EXTENSIONS: u8 = 8;
pub type Eval = i32;

pub const NULLMOVE: Move = Move(0);

pub const INFINITY: Eval = 30_000;

pub struct ThreadData {
    state: GameState,
    max_depth: u8,
    trans_table: Arc<SyncUnsafeCell<LockLessTransTable>>,
    thread_num: usize,
    search_info: SearchInfo,
}

pub struct SearchInfo {
    start_time: Instant,
    max_time: Duration,
    killer_table: KillerTable,
    history_table: [[[u32; NUM_OF_SQUARES]; NUM_OF_SQUARES]; NUM_OF_PLAYERS],
    stop_flag: Arc<SyncUnsafeCell<bool>>,
    search_depth: u8,
}

impl SearchInfo {
    fn new(max_time: Duration, stop_flag: Arc<SyncUnsafeCell<bool>>) -> Self {
        SearchInfo { start_time: Instant::now(), max_time, killer_table: Default::default(), history_table: [[[0; NUM_OF_SQUARES]; NUM_OF_SQUARES]; NUM_OF_PLAYERS], stop_flag, search_depth: 0 }
    }

    fn time_over(&self) -> bool {
        self.start_time.elapsed() > self.max_time
    }
}

#[derive(Default)]
struct KillerTable([[Move; MAX_KILLER_MOVES]; MAX_SEARCH_DEPTH as usize]);

impl KillerTable {
    fn store_killer(&mut self, depth: u8, r#move: Move) {
        if self.0[depth as usize][0] != r#move {
            self.0[depth as usize][1] = self.0[depth as usize][0];
            self.0[depth as usize][0] = r#move;
        }
    }

    fn is_killer(&mut self, depth: u8, r#move: Move) -> Option<u8> {
        if self.0[depth as usize][0] == r#move {
            return Some(0);
        }
        if self.0[depth as usize][1] == r#move {
            return Some(1);
        }
        None
    } 
}

#[derive(ConstParamTy, PartialEq, Eq)]
pub enum SearchProtocol {
    Uci(UciMode),
    Texel,
    Debug,
}

#[derive(ConstParamTy, PartialEq, Eq)]
pub enum UciMode {
    Infinite,
    Movetime,
}

pub fn search<const SEARCHMODE: SearchProtocol>(threads: usize, max_time: Duration, state: GameState, stop_flag: Arc<SyncUnsafeCell<bool>>, max_depth: u8, trans_table: Arc<SyncUnsafeCell<LockLessTransTable>>) -> (Move, Eval) {
    if let Some(entry) = OPENING_BOOK.get(&state.to_reduced_book_fen()) {
            
        let moves: Vec<String> = entry.iter().map(|item| item.0.to_string()).collect();
        let weights: Vec<u32> = entry.iter().map(|item| item.1).collect(); 
        let dist = WeightedIndex::new(weights).unwrap();
        let mut rng = thread_rng();
        let r#move = moves[dist.sample(&mut rng)].to_string();
        if let SearchProtocol::Uci(_) = SEARCHMODE {
            println!("bestmove {}", r#move);
        }
        
        unsafe {
            let stop_flag_ptr = stop_flag.get();
            *stop_flag_ptr = true;
        }
        return (Move::from_text_move(&state, &r#move), 0);
    }
    
    let mut thread_pool = vec![];
    for thread in 0..threads {
        let state_clone = state.clone();
        let trans_table_clone = Arc::clone(&trans_table);
        let stop_flag_clone = Arc::clone(&stop_flag);
        thread_pool.push(thread::spawn(move || {
            let thread_data = ThreadData {
                state: state_clone,
                max_depth,
                trans_table: trans_table_clone,
                thread_num: thread,
                search_info: SearchInfo::new(max_time, stop_flag_clone)
            };
            iterative_deepening::<SEARCHMODE>(thread_data)
        }));
    }
    let mut results = Vec::new();
    for thread in thread_pool {
        results.push(thread.join().unwrap());
    }
    assert!(results[0].0 != NULLMOVE);
    unsafe {
        (*trans_table.get()).advance_age()
    }
    results[0]
}

fn extract_pv(state: &GameState, t_table: *mut LockLessTransTable) -> Vec<Move> {
    let mut moves = Vec::new();
    let mut copy_state = state.clone();
    unsafe {
        let mut depth = 0;
        while let Some(entry) = (*t_table).get(copy_state.zobrist) {
            if depth > entry.depth() {
                break;
            }
            let pvmove = entry.best_move();
            moves.push(pvmove);
            copy_state.apply_legal_move(pvmove);
            depth += 1;
        }
    }

    moves
}

fn pv_to_string(pv: &Vec<Move>) -> String {
    let mut string = String::new();
    for p in pv {
        string.push_str(&format!("{} ", p.to_algebraic()));
    }
    string
}

pub fn iterative_deepening<const SEARCHMODE: SearchProtocol>(mut thread_data: ThreadData) -> (Move, Eval) {
    let mut best_move = NULLMOVE;
    let mut best_eval: i32 = -INFINITY;
    for depth in 1..thread_data.max_depth {
        thread_data.search_info.search_depth = depth;
        let score = alpha_beta::<SEARCHMODE>(-INFINITY, INFINITY, &mut thread_data.state, depth, &mut thread_data.search_info, thread_data.trans_table.get(), true, 0);
        
        // If the search stopped prematurely we want to not return the value of it so we break
        unsafe {
            if ((SEARCHMODE == SearchProtocol::Uci(UciMode::Movetime) || SEARCHMODE == SearchProtocol::Texel) && thread_data.search_info.time_over()) || *thread_data.search_info.stop_flag.get() {
                break;
            }
        }
        best_eval = score;
        let pv = extract_pv(&thread_data.state, thread_data.trans_table.get());
        best_move = pv[0];
        if let SearchProtocol::Uci(_) = SEARCHMODE && thread_data.thread_num == 0 {
            let pv_string = pv_to_string(&pv);
            println!("info depth {} score cp {} pv {}", depth, best_eval, pv_string);
        }
    }
    if thread_data.thread_num == 0 {
        unsafe {
            *thread_data.search_info.stop_flag.get() = true;
        }
        if let SearchProtocol::Uci(_) = SEARCHMODE {
            println!("bestmove {}", best_move.to_algebraic());
        }
    }
    (best_move, best_eval)
}

fn alpha_beta<const SEARCHMODE: SearchProtocol>(alpha: Eval, beta: Eval, state: &mut GameState, depth: u8, search_info: &mut SearchInfo, trans_table: *mut LockLessTransTable, do_null: bool, check_extensions: u8) -> Eval {
    if depth == 0 {
        return quiescent_search::<SEARCHMODE>(state, alpha, beta, MAX_QUIESCENT_DEPTH, search_info)
    }

    if state.has_repitition() || state.fifty_move_rule >= 100 {
        return 0;
    }

    let mut depth = depth;
    let mut check_extensions = check_extensions;
    let in_check = state.is_in_check();
    
    if in_check && check_extensions <= MAX_CHECK_EXTENSIONS {
        depth += 1;
        check_extensions += 1;
    }

    let mut alpha = alpha;
    let original_alpha = alpha;

    let mut pvmove = NULLMOVE;

    unsafe {
        if let Some(entry) = (*trans_table).get(state.zobrist) {
            pvmove = entry.best_move();
            assert!(pvmove != NULLMOVE);
            if entry.depth() >= depth {
                match entry.flag() {
                    LockLessFlag::Alpha => {
                        if entry.value() <= alpha {
                            return alpha;
                        }
                    },
                    LockLessFlag::Beta => {
                        if entry.value() >= beta {
                            return beta;
                        }
                    },
                    LockLessFlag::Exact => {
                        return entry.value();
                    }
                }
            }
        }
    }

    // Null-Move heuristic
    if do_null && depth >= 4 && !in_check && state.phase() <= 220 && state.plys != 0 {
        state.make_null_move();
        let null_move_value = -alpha_beta::<SEARCHMODE>(-beta, -beta + 1, state, depth - 4, search_info, trans_table, false, check_extensions);
        state.undo_null_move();
        if null_move_value >= beta && null_move_value.abs() < INFINITY {
            return beta;
        }
    }


    let mut legals = 0;

    let mut moves = state.generate_pseudo_legal_moves();

    moves.value_moves(search_info, depth, state.side_to_move());

    if pvmove != NULLMOVE {
        for move_index in 0..moves.length {
            if moves.moves[move_index as usize] == pvmove {
                moves.values[move_index as usize] = u32::MAX;
            }
        }
    }

    let mut best_move = NULLMOVE;
    let mut best_value = -INFINITY;
    for move_index in 0..moves.length {
        moves.highest_next_to_index(move_index);
        let r#move = moves.moves[move_index as usize];
        if !state.apply_pseudo_legal_move(r#move) {
            continue;
        }
        legals += 1;
        let value = if depth > 3 && legals > 3 && !r#move.is_capture() && !r#move.is_promotion() && !in_check && search_info.killer_table.is_killer(depth, r#move).is_none() {
            let reduction = if legals > 6 { 2 } else { 1 };
            let tmp_value = -alpha_beta::<SEARCHMODE>(-beta, -alpha, state, depth - 1 - reduction, search_info, trans_table, true, check_extensions);
            if tmp_value > alpha {
                -alpha_beta::<SEARCHMODE>(-beta, -alpha, state, depth - 1, search_info, trans_table, true, check_extensions)
            } else {
                tmp_value
            }
        } else {
            -alpha_beta::<SEARCHMODE>(-beta, -alpha, state, depth - 1, search_info, trans_table, true, check_extensions)
        };
        state.undo_move();
        unsafe {
            if (SEARCHMODE == SearchProtocol::Uci(UciMode::Movetime) || SEARCHMODE == SearchProtocol::Texel) && search_info.time_over() {
                return alpha;
            }
            if *search_info.stop_flag.get() {
                return alpha;
            }
        }
        if value > best_value {
            best_value = value;
            best_move = r#move;
            if value > alpha {
                if value >= beta {

                    if !r#move.is_capture() {
                        search_info.killer_table.store_killer(depth, r#move);
                    }

                    unsafe {
                        assert!(best_move != NULLMOVE);
                        (*trans_table).insert(state.zobrist, LockLessValue::new(best_move, LockLessFlag::Beta, beta, depth))
                    }

                    return beta;
                }
                alpha = value;
                if !r#move.is_capture() {
                    search_info.history_table[state.side_to_move()][r#move.from()][r#move.to()] += depth as u32;
                }
            }
        }
        
    }

    if legals == 0 {
        if in_check {
            return -INFINITY;
        } 
        else {
            return 0;
        }
    }

    // If all moves suck the same we just give back the first viable move as the best move.
    if best_move == NULLMOVE {
        for index in 0..moves.length {
            if state.apply_pseudo_legal_move(moves.moves[index as usize]) {
                state.undo_move();
                best_move = moves.moves[index as usize];
                break;
            }
        }
    }

    assert!(best_move != NULLMOVE);
    unsafe {
        if alpha != original_alpha {
            (*trans_table).insert(state.zobrist, LockLessValue::new(best_move, LockLessFlag::Exact, best_value, depth));
        } else {
            (*trans_table).insert(state.zobrist, LockLessValue::new(best_move, LockLessFlag::Alpha, best_value, depth));
        }
    }

    alpha
}

fn quiescent_search<const SEARCHMODE: SearchProtocol>(state: &mut GameState, alpha: Eval, beta: Eval, depth: u8, search_info: &mut SearchInfo) -> Eval {
    
    if state.has_repitition() || state.fifty_move_rule >= 100 {
        return alpha;
    }

    let stand_pat: Eval = state.static_eval();
    if depth == 0 {
        return stand_pat;
    }
    let mut alpha = alpha;
    if stand_pat >= beta {
        return beta;
    }
    if alpha < stand_pat {
        alpha = stand_pat;
    }

    let mut orderd_moves = state.generate_pseudo_legal_captures();
    orderd_moves.value_moves_mvv_lva();
    for move_index in 0..orderd_moves.length {
        orderd_moves.highest_next_to_index(move_index);
        let r#move = orderd_moves.moves[move_index as usize];
        if !state.apply_pseudo_legal_move(r#move) {
            continue;
        }
        let score = -quiescent_search::<SEARCHMODE>(state, -beta, -alpha, depth - 1, search_info);
        state.undo_move();
        unsafe {
            if ((SEARCHMODE == SearchProtocol::Uci(UciMode::Movetime) || SEARCHMODE == SearchProtocol::Texel) && search_info.time_over()) || *search_info.stop_flag.get() {
                return alpha;
            }
        }
        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }
    alpha
}

static MVV_LVA: [[u32; NUM_OF_PIECES]; NUM_OF_PIECES] = [
    [15, 12, 14, 13, 11, 10], // Victim Pawn, P, R, K, B, Q, K
    [45, 42, 44, 43, 41, 40], // Victim Rook
    [25, 22, 24, 23, 21, 20], // Victim Knight
    [35, 32, 34, 33, 31, 30], // Victim Bishop
    [55, 52, 54, 53, 51, 50], // Victim Queen
    [0, 0, 0, 0, 0, 0], // Victim King
];

const MVV_LVA_VALUE: u32 = u32::MAX - 1000;
const KILLER_VALUE: u32 = MVV_LVA_VALUE - 1000;
const SECONDARY_KILLER_VALUE: u32 = KILLER_VALUE - 1000;

impl MoveList {
    fn value_moves(&mut self, search_info: &mut SearchInfo, depth: u8, side_to_move: Side) {
        for move_index in 0..self.length {
            let r#move = self.moves[move_index as usize];
            if r#move.is_capture() {
                self.values[move_index as usize] += MVV_LVA_VALUE + MVV_LVA[r#move.captured_piece()][r#move.moving_piece()];
            } else if let Some(index) = search_info.killer_table.is_killer(depth, r#move) {
                if index == 0 {
                    self.values[move_index as usize] += KILLER_VALUE;
                } else if index == 1 {
                    self.values[move_index as usize] += SECONDARY_KILLER_VALUE;
                }
            } else {
                self.values[move_index as usize] += search_info.history_table[side_to_move][r#move.from()][r#move.to()];
                assert!(self.values[move_index as usize] < SECONDARY_KILLER_VALUE);
            }
        }
    }

    fn value_moves_mvv_lva(&mut self) {
        for move_index in 0..self.length {
            let r#move = self.moves[move_index as usize];
            if r#move.is_capture() {
                self.values[move_index as usize] += MVV_LVA_VALUE + MVV_LVA[r#move.captured_piece()][r#move.moving_piece()];
            }
        }
    }
    fn highest_next_to_index(&mut self, start_index: u8) {
        for index in (start_index + 1)..self.length {
            if self.values[index as usize] > self.values[start_index as usize] {
                self.swap(start_index, index);
            }
        }
    }

    fn swap(&mut self, index1: u8, index2: u8) {
        let temp_value = self.values[index1 as usize];
        let temp_move = self.moves[index1 as usize];
        self.values[index1 as usize] = self.values[index2 as usize];
        self.moves[index1 as usize] = self.moves[index2 as usize];
        self.values[index2 as usize] = temp_value;
        self.moves[index2 as usize] = temp_move;
    }
}