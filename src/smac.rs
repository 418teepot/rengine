use std::{fs, io::Write, env};
use clap::Parser;

use crate::{smpsearch::Eval, eval::{EvalParams, EVAL_PARAMS}, gamestate::{PAWN, ROOK, KNIGHT, BISHOP, QUEEN}, texel::{read_texel_sample_file, mean_square_error, K}};

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    seed: i64,
    #[arg(long)]
    mp: Eval,
    #[arg(long)]
    ep: Eval,
    #[arg(long)]
    mr: Eval,
    #[arg(long)]
    er: Eval,
    #[arg(long)]
    mn: Eval,
    #[arg(long)]
    en: Eval,
    #[arg(long)]
    mb: Eval,
    #[arg(long)]
    eb: Eval,
    #[arg(long)]
    mq: Eval,
    #[arg(long)]
    eq: Eval,
    
    #[arg(long)]
    mip: Eval,
    #[arg(long)]
    eip: Eval,
    #[arg(long)]
    msb: Eval,
    #[arg(long)]
    esb: Eval,
    #[arg(long)]
    mdp: Eval,
    #[arg(long)]
    edp: Eval,
}

pub fn smac() -> std::io::Result<()> {
    let args = Args::parse();

    unsafe {
        EVAL_PARAMS.mg_piece_value[PAWN] = args.mp;
        EVAL_PARAMS.mg_piece_value[ROOK] = args.mr;
        EVAL_PARAMS.mg_piece_value[KNIGHT] = args.mn;
        EVAL_PARAMS.mg_piece_value[BISHOP] = args.mb;
        EVAL_PARAMS.mg_piece_value[QUEEN] = args.mq;
        
        EVAL_PARAMS.eg_piece_value[PAWN] = args.ep;
        EVAL_PARAMS.eg_piece_value[ROOK] = args.er;
        EVAL_PARAMS.eg_piece_value[KNIGHT] = args.en;
        EVAL_PARAMS.eg_piece_value[BISHOP] = args.eb;
        EVAL_PARAMS.eg_piece_value[QUEEN] = args.eq;

        EVAL_PARAMS.mg_doubled_penalty = args.mdp;
        EVAL_PARAMS.eg_doubled_penalty = args.edp;
        EVAL_PARAMS.mg_isolated_penalty = args.mip;
        EVAL_PARAMS.eg_isolated_penalty = args.eip;
        EVAL_PARAMS.mg_supported_bonus = args.msb;
        EVAL_PARAMS.eg_supported_bonus = args.esb;
        
    }
    let fen_and_values = read_texel_sample_file();
    let mut best_e = mean_square_error(K, &fen_and_values);
    println!("cost={}", best_e);
    Ok(())
}