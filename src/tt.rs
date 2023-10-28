use std::{default, io::Empty};

use crate::{zobrist::ZobristHash, search::Eval, r#move::Move};

#[derive(Debug, Default, Clone)]
pub struct TranspositionTable {
    pub buckets: Vec<TTEntry>,
    pub size: usize,
}

impl TranspositionTable {
    pub fn new(size: usize) -> Self {
        TranspositionTable { buckets: vec![TTEntry::default(); size], size }
    }

    pub fn insert(&mut self, key: ZobristHash, value: Eval, flag: TTEntryFlag, depth: u8, best_move: Move) {
        let index = self.index(key);
        self.buckets[index] = TTEntry {
            zobrist_key: key,
            value,
            flag,
            depth,
            best_move
        }
    }

    #[inline(always)]
    pub fn index(&self, key: ZobristHash) -> usize {
        (key.0 % self.size as u64) as usize 
    }

    pub fn probe(&self, key: ZobristHash) -> Option<TTEntry> {
        let index = self.index(key);
        if self.buckets[index].flag != TTEntryFlag::Empty && self.buckets[index].zobrist_key == key {
            return Some(self.buckets[index])
        }
        None
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum TTEntryFlag {
    Exact,
    Alpha,
    Beta,
    #[default]
    Empty,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct TTEntry {
    pub zobrist_key: ZobristHash,
    pub value: Eval,
    pub flag: TTEntryFlag,
    pub depth: u8,
    pub best_move: Move,
}

