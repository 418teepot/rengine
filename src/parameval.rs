use std::{sync::Mutex, cell::SyncUnsafeCell};

use crate::{smpsearch::Eval, bitboard::NUM_OF_SQUARES, gamestate::{NUM_OF_PIECES, NUM_OF_PLAYERS}};

#[derive(Debug, Clone)]
struct EvalParams {
    mg_piece_values: [Eval; 7],
    eg_piece_values: [Eval; 7],
    mg_psqt_values: [[[Eval; NUM_OF_SQUARES]; NUM_OF_PIECES]; NUM_OF_PLAYERS],
    eg_psqt_values: [[[Eval; NUM_OF_SQUARES]; NUM_OF_PIECES]; NUM_OF_PLAYERS],
    tempo_value: Eval,
    mg_rook_mobility: [Eval; 15],
    mg_bishop_mobility: [Eval; 14],
    mg_knight_mobility: [Eval; 9],
    mg_queen_mobility: [Eval; 28],
    eg_knight_mobility: [Eval; 9],
    eg_queen_mobility: [Eval; 28],
    eg_bishop_mobility: [Eval; 14],
    eg_rook_mobility: [Eval; 15],
    king_non_castled_penalty: Eval,
    missing_pawn_shield_penalty: Eval,
    mg_rook_open_file_bonus: [Eval; 3],
    eg_rook_open_file_bonus: [Eval; 3],
    mg_pawn_doubled_penalty: Eval,
    mg_pawn_isolated_penalty: Eval,
    mg_passed_bonuses: [Eval; 8],
    mg_connected_bonus: Eval,
    eg_pawn_doubled_penalty: Eval,
    eg_pawn_isolated_penalty: Eval,
    eg_passed_bonuses: [Eval; 8],
    eg_connected_bonus: Eval,
}

lazy_static! {
    static ref CURRENT_PARAMS: SyncUnsafeCell<EvalParams> = {
        SyncUnsafeCell::new(EvalParams { 
            mg_piece_values: todo!(), 
            eg_piece_values: todo!(), 
            mg_psqt_values: todo!(), 
            eg_psqt_values: todo!(), 
            tempo_value: todo!(), 
            mg_rook_mobility: todo!(), 
            mg_bishop_mobility: todo!(), 
            mg_knight_mobility: todo!(), 
            mg_queen_mobility: todo!(), 
            eg_knight_mobility: todo!(), 
            eg_queen_mobility: todo!(), 
            eg_bishop_mobility: todo!(), 
            eg_rook_mobility: todo!(), 
            king_non_castled_penalty: todo!(), 
            missing_pawn_shield_penalty: todo!(), 
            mg_rook_open_file_bonus: todo!(), 
            eg_rook_open_file_bonus: todo!(), 
            mg_pawn_doubled_penalty: todo!(), 
            mg_pawn_isolated_penalty: todo!(), 
            mg_passed_bonuses: todo!(), 
            mg_connected_bonus: todo!(), 
            eg_pawn_doubled_penalty: todo!(), 
            eg_pawn_isolated_penalty: todo!(), 
            eg_passed_bonuses: todo!(), 
            eg_connected_bonus: todo!() 
        })
    };
}