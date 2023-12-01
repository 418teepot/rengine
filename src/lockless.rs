use crate::search::INFINITY;

use crate::{zobrist::ZobristHash, r#move::Move, search::Eval};

struct LockLessTransTable<const SIZE: usize> {
    buckets: [LockLessEntry; SIZE],
}

impl<const SIZE: usize> LockLessTransTable<SIZE> {
    fn insert(&mut self, key: ZobristHash, value: LockLessValue) {
        let index = key.0 as usize % SIZE;
        self.buckets[index].key = ZobristHash(key.0 ^ value.0);
        self.buckets[index].value = value;
    }

    fn get(&self, key: ZobristHash) -> Option<LockLessValue> {
        let index = key.0 as usize % SIZE;
        if self.buckets[index].value.0 != 0 && self.buckets[index].key.0 ^ self.buckets[index].value.0 == key.0 {
            return Some(self.buckets[index].value);
        }
        None
    }

    fn new() -> Self {
        LockLessTransTable {
            buckets: [LockLessEntry::default(); SIZE],
        }
    }
}

unsafe impl<const SIZE: usize> Sync for LockLessTransTable<SIZE> {}

#[derive(Copy, Clone, Default)]
struct LockLessEntry {
    key: ZobristHash,
    value: LockLessValue,
}

#[derive(Copy, Clone, Default)]
struct LockLessValue(pub u64);

impl LockLessValue {
    pub fn new(r#move: Move, flag: LockLessFlag, value: Eval, depth: u8) -> Self {
        LockLessValue((value + INFINITY) as u64 | (depth as u64) << 16 | (flag as u64) << 23 | (r#move.0 as u64) << 25)
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