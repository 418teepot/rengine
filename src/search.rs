use std::cmp::{max, min};
use std::time::{Duration, Instant};

use crate::gamestate::{GameState, NUM_OF_PIECES, BLACK, Side, KING};
use crate::r#move::{Move, MoveList};
use crate::tt::TranspositionTable;

pub type Eval = i32;

const INFINITY: Eval = 2000000000;
const MAX_QUIESCENCE: u8 = 7;

pub const MAX_GAME_DEPTH: Eval = 512;
pub const IS_MATE: Eval = INFINITY - MAX_GAME_DEPTH;

const MAX_SEARCH_DEPTH: u8 = 30;
const MAX_KILLER_MOVES: usize = 2;

const TRANS_TABLE_SIZE: usize = 100_000;

pub struct SearchInfo {
    pub killer_table: KillerTable,
    pub trans_table: TranspositionTable,
    pub search_data: SearchData,
    pub begin: Instant,
    pub max_time: Duration,
}

#[derive(Default)]
pub struct KillerTable([[Move; MAX_KILLER_MOVES]; MAX_SEARCH_DEPTH as usize]);

impl SearchInfo {
    fn new(begin: Instant, max_time: Duration) -> Self {
        SearchInfo {
            killer_table: KillerTable::default(),
            trans_table: TranspositionTable::new(TRANS_TABLE_SIZE),
            search_data: SearchData::default(),
            begin,
            max_time
        }
    }

    fn time_over(&self) -> bool {
        self.begin.elapsed() > self.max_time
    }
}

impl KillerTable {
    fn store_killer(&mut self, depth: u8, r#move: Move) {
        if self.0[depth as usize][0] != r#move {
            self.0[depth as usize][1] = self.0[depth as usize][0];
            self.0[depth as usize][0] = r#move;
        }
    }

    fn is_killer(&mut self, depth: u8, r#move: Move) -> bool {
        if self.0[depth as usize][0] == r#move || self.0[depth as usize][1] == r#move {
            return true;
        }
        false
    } 
}

#[derive(Default)]
pub struct SearchData {
    pub cut_nodes: u32,
    pub hash_hits: u32,
    pub nodes_visited: u32,
}

pub fn iterative_deepening_timed(state: &mut GameState, max_time: Duration) -> (Move, Eval) {
    let mut search_info = SearchInfo::new(Instant::now(), max_time);
    let begin = Instant::now();
    let mut best_move = Move::new_from_to(0, 0, 0);
    let mut best_eval = -INFINITY;
    let mut depth = 1;
    loop {
        let (candidate_move, candidate_eval) = pick_best_move_timed(state, begin, max_time, depth, &mut search_info);
        if search_info.time_over() {
            break;
        }
        (best_move, best_eval) = (candidate_move, candidate_eval);
        /* 
        print!("info depth {depth} ");
        print!("eval {} ", eval_into_white_viewpoint(best_eval, state.side_to_move()));
        print!("tthits {} ", search_info.search_data.hash_hits);
        // print!("cutnodes {} ", search_info.search_data.cut_nodes);
        // print!("visited {} ", search_info.search_data.nodes_visited);
        println!("cutratio {:.2} ", (search_info.search_data.cut_nodes as f64 / search_info.search_data.nodes_visited as f64) * 100.0_f64);
        */

        search_info.search_data.hash_hits = 0;
        search_info.search_data.cut_nodes = 0;
        search_info.search_data.nodes_visited = 0;
        depth+=1;
        if depth == MAX_SEARCH_DEPTH {
            break;
        }
    }
    (best_move, best_eval)
}

pub fn pick_best_move_timed(state: &mut GameState, begin: Instant, max_time: Duration, depth: u8, search_info: &mut SearchInfo) -> (Move, Eval) {
    let mut best_move = Move::new_from_to(0, 0, 0);
    let mut best_val = -INFINITY;
    let mut orderd_moves = state.generate_legal_moves();
    orderd_moves.value_moves(search_info, depth);
    for move_index in 0..orderd_moves.length {
        orderd_moves.highest_next_to_index(move_index);
        let r#move = orderd_moves.moves[move_index as usize];
        if !state.apply_pseudo_legal_move(r#move) {
            continue;
        }
        let value_candidate = -alpha_beta_timed(state, -INFINITY, INFINITY, depth, search_info, true);
        state.undo_move();
        if search_info.time_over() {
            return (best_move, best_val);
        }
        if value_candidate > best_val {
            best_val = value_candidate;
            best_move = r#move;
        }
        
    }
    (best_move, best_val)
}

pub fn alpha_beta_timed(state: &mut GameState, alpha: Eval, beta: Eval, depth: u8, search_info: &mut SearchInfo, do_null: bool) -> Eval {
    search_info.search_data.nodes_visited += 1;
    if search_info.time_over() {
        return alpha;
    }
    if depth == 0 {
        return quiescent_search_fixed(state, alpha, beta, MAX_QUIESCENCE, search_info);
    }
    
    
    if state.has_repitition() || state.fifty_move_rule >= 100 {
        return 0;
    }

    let mut depth = depth;
    let in_check = state.is_in_check();
    if in_check {
        depth += 1;
    }

    let mut alpha = alpha;
    let original_alpha = alpha;

    let mut pvmove = Move::new_from_to(0, 0, 0);

    if let Some(entry) = search_info.trans_table.probe(state.zobrist) {
        pvmove = entry.best_move;
        if entry.depth >= depth {
            search_info.search_data.hash_hits += 1;
            match entry.flag {
                crate::tt::TTEntryFlag::Alpha => {
                    if entry.value <= alpha {
                        return alpha;
                    }
                },
                crate::tt::TTEntryFlag::Beta => {
                    if entry.value >= beta {
                        return beta;
                    }
                },
                crate::tt::TTEntryFlag::Exact => {
                    return entry.value;
                },
                _ => unreachable!(),
            }
        }
    }
    
    let mut legals = 0;

    let mut moves = state.generate_pseudo_legal_moves();

    moves.value_moves(search_info, depth);

    if pvmove != Move::new_from_to(0, 0, 0) {
        for move_index in 0..moves.length {
            if moves.moves[move_index as usize] == pvmove {
                moves.values[move_index as usize] = INFINITY;
            }
        }
    }

    let mut best_move = Move::new_from_to(0, 0, 0);
    let mut best_value = -INFINITY;
    let mut value = -INFINITY;
    for move_index in 0..moves.length {
        moves.highest_next_to_index(move_index);
        let r#move = moves.moves[move_index as usize];
        if !state.apply_pseudo_legal_move(r#move) {
            continue;
        }
        legals += 1;
        value = -alpha_beta_timed(state, -beta, -alpha, depth - 1, search_info, do_null);
        /* 
        if r#move == pvmove {
            value = -alpha_beta_timed(state, -beta, -alpha, depth - 1, search_info, do_null);
        } else {
            value = -alpha_beta_timed(state, -alpha - 1, -alpha, depth - 1, search_info, do_null);
            if alpha < value && value < beta {
                value = -alpha_beta_timed(state, -beta, -alpha, depth - 1, search_info, do_null)
            }
        };
        */
        state.undo_move();
        if value > best_value {
            best_value = value;
            best_move = r#move;
            if value > alpha {
                if value >= beta {
                    search_info.search_data.cut_nodes += 1;

                    if !r#move.is_capture() {
                        search_info.killer_table.store_killer(depth, r#move);
                    }

                    search_info.trans_table.insert(state.zobrist, beta, crate::tt::TTEntryFlag::Beta, depth, best_move);

                    return beta;
                }
                alpha = value;
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


    if alpha != original_alpha {
        search_info.trans_table.insert(state.zobrist, best_value, crate::tt::TTEntryFlag::Exact, depth, best_move);
    } else {
        search_info.trans_table.insert(state.zobrist, alpha, crate::tt::TTEntryFlag::Alpha, depth, best_move);
    }

    alpha
}

pub fn quiescent_search_fixed(state: &mut GameState, alpha: Eval, beta: Eval, depth: u8, search_info: &mut SearchInfo) -> Eval {
    search_info.search_data.nodes_visited += 1;
    if state.has_repitition() || state.fifty_move_rule >= 100 {
        return 0;
    }

    let stand_pat: Eval = state.static_eval();
    if depth == 0 {
        return stand_pat;
    }
    let mut alpha = alpha;
    if stand_pat >= beta {
        search_info.search_data.cut_nodes += 1;
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
        let score = -quiescent_search_fixed(state, -beta, -alpha, depth - 1, search_info);
        state.undo_move();

        if score >= beta {
            search_info.search_data.cut_nodes += 1;
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }

    alpha
}

pub fn eval_into_white_viewpoint(value: Eval, side_to_move: Side) -> f64 {
    if side_to_move == BLACK {
        -value as f64 / 100_f64
    } else {
        value as f64 / 100_f64
    }
} 

static MVV_LVA: [[Eval; NUM_OF_PIECES]; NUM_OF_PIECES] = [
    [15, 12, 14, 13, 11, 10], // Victim Pawn, P, R, K, B, Q, K
    [45, 42, 44, 43, 41, 40], // Victim Rook
    [25, 22, 24, 23, 21, 20], // Victim Knight
    [35, 32, 34, 33, 31, 30], // Victim Bishop
    [55, 52, 54, 53, 51, 50], // Victim Queen
    [0, 0, 0, 0, 0, 0], // Victim King
];

const MVV_LVA_VALUE: Eval = 1_000;
const KILLER_VALUE: Eval = 100;

impl MoveList {
    pub fn value_moves(&mut self, search_info: &mut SearchInfo, depth: u8) {
        for move_index in 0..self.length {
            let r#move = self.moves[move_index as usize];
            if r#move.is_capture() {
                self.values[move_index as usize] += MVV_LVA_VALUE + MVV_LVA[r#move.captured_piece()][r#move.moving_piece()];
            } else if search_info.killer_table.is_killer(depth, r#move) {
                self.values[move_index as usize] += KILLER_VALUE;
            }
        }
    }

    pub fn value_moves_mvv_lva(&mut self) {
        for move_index in 0..self.length {
            let r#move = self.moves[move_index as usize];
            if r#move.is_capture() {
                self.values[move_index as usize] += MVV_LVA_VALUE + MVV_LVA[r#move.captured_piece()][r#move.moving_piece()];
            }
        }
    }
    pub fn highest_next_to_index(&mut self, start_index: u8) {
        for index in (start_index + 1)..self.length {
            if self.values[index as usize] > self.values[start_index as usize] {
                self.swap(start_index, index);
            }
        }
    }

    pub fn swap(&mut self, index1: u8, index2: u8) {
        let temp_value = self.values[index1 as usize];
        let temp_move = self.moves[index1 as usize];
        self.values[index1 as usize] = self.values[index2 as usize];
        self.moves[index1 as usize] = self.moves[index2 as usize];
        self.values[index2 as usize] = temp_value;
        self.moves[index2 as usize] = temp_move;
    }
}