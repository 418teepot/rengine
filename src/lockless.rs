use crate::{zobrist::ZobristHash, r#move::Move};

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

#[derive(Copy, Clone, Default)]
struct LockLessEntry {
    key: ZobristHash,
    value: LockLessValue,
}

#[derive(Copy, Clone, Default)]
struct LockLessValue(pub u64);
