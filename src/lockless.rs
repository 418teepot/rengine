use crate::smpsearch::INFINITY;

use crate::{zobrist::ZobristHash, r#move::Move, smpsearch::Eval};

const TRANS_TABLE_SIZE: usize = 1_000_000;

pub struct LockLessTransTable {
    buckets: [LockLessEntry; TRANS_TABLE_SIZE],
    ages: [u8; TRANS_TABLE_SIZE],
    current_age: u8,
}

impl LockLessTransTable {
    pub fn insert(&mut self, key: ZobristHash, value: LockLessValue) {
        let index = key.0 as usize % self.buckets.len();
        let old_val = self.buckets[index];
        if old_val.key.0 == 0
        || self.ages[index] < self.current_age
        || old_val.value.depth() <= value.depth() {
            self.buckets[index].key = ZobristHash(key.0 ^ value.0);
            self.buckets[index].value = value;
            self.ages[index] = self.current_age;
        }
    }

    pub fn get(&self, key: ZobristHash) -> Option<LockLessValue> {
        let index = key.0 as usize % self.buckets.len();
        if self.buckets[index].value.0 != 0 && self.buckets[index].key.0 ^ self.buckets[index].value.0 == key.0 {
            return Some(self.buckets[index].value);
        }
        None
    }

    pub fn new() -> Self {
        LockLessTransTable {
            buckets: [LockLessEntry::default(); TRANS_TABLE_SIZE],
            ages: [0; TRANS_TABLE_SIZE],
            current_age: 0,
        }
    }

    pub fn clear(&mut self) {
        for i in 0..TRANS_TABLE_SIZE {
            self.buckets[i] = Default::default();
            self.ages[i] = 0;
        }
        self.current_age = 0;
    }

    pub fn advance_age(&mut self) {
        self.current_age += 1;
    }
}

unsafe impl Sync for LockLessTransTable {}

#[derive(Copy, Clone, Default)]
struct LockLessEntry {
    key: ZobristHash,
    value: LockLessValue,
}

#[derive(Copy, Clone, Default)]
pub struct LockLessValue(pub u64);

impl LockLessValue {
    pub fn new(r#move: Move, flag: LockLessFlag, value: Eval, depth: u8) -> Self {
        LockLessValue((value + INFINITY) as u64  | (depth as u64) << 16 | (flag as u64) << 23 | (r#move.0 as u64) << 25)
    }

    pub fn value(&self) -> Eval {
        (self.0 & 0xFFFF) as Eval - INFINITY
    }

    pub fn depth(&self) -> u8 {
        ((self.0 >> 16) & 0x3F) as u8
    }

    pub fn flag(&self) -> LockLessFlag {
        ((self.0 >> 23) & 0x3).into()
    }

    pub fn best_move(&self) -> Move {
        Move((self.0 >> 25) as u32)
    }
}

pub enum LockLessFlag {
    Alpha,
    Beta,
    Exact
}

impl From<LockLessFlag> for u64 {
    #[inline(always)]
    fn from(value: LockLessFlag) -> Self {
        match value {
            LockLessFlag::Alpha => 0,
            LockLessFlag::Beta => 1,
            LockLessFlag::Exact => 2,
        }
    }
}

impl From<u64> for LockLessFlag {
    fn from(value: u64) -> Self {
        match value {
            0 => LockLessFlag::Alpha,
            1 => LockLessFlag::Beta,
            2 => LockLessFlag::Exact,
            _ => unreachable!(),        
        }
    }
}