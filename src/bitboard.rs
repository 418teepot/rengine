use std::ops::{BitOrAssign, BitOr, BitXorAssign, BitAnd};

#[derive(PartialEq, Default, Copy, Clone, Debug)]
pub struct Bitboard(pub u64);

pub type Square = usize;

pub const NUM_OF_SQUARES: usize = 64;

impl Bitboard {
    pub fn from_squares(squares: &[Square]) -> Self {
        let mut bitboard = Bitboard(0);
        for square in squares {
            bitboard |= Bitboard(1 << square);
        }
        bitboard
    }

    pub fn square(square: Square) -> Self {
        Bitboard(1 << square)
    }

    pub fn empty() -> Self {
        Bitboard(0)
    }

    pub fn add_piece(&mut self, square: Square) {
        self.0 |= 1 << square;
    } 

    // TODO: Secure xor wihtout having to check if square is actually set?
    pub fn remove_piece(&mut self, square: Square) {
        self.0 &= !(1 << square);
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub fn next_piece_index(&self) -> Square {
        self.0.trailing_zeros() as Square
    }
}

impl BitOr for Bitboard {
    type Output = Bitboard;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 = self.0 | rhs.0;
    }
}

impl BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 = self.0 ^ rhs.0;
    }
}

pub struct BitboardIterator {
    bitset: Bitboard,
}

impl Iterator for BitboardIterator {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bitset == Bitboard(0) {
            return None;
        }
        let index = self.bitset.next_piece_index();
        self.bitset ^= Bitboard(1 << index);
        Some(index)
    }
}

impl IntoIterator for Bitboard {
    type Item = Square;

    type IntoIter = BitboardIterator;

    fn into_iter(self) -> Self::IntoIter {
        BitboardIterator {
            bitset: self,
        }
    }
}

impl BitAnd for Bitboard {
    type Output = Bitboard;

    fn bitand(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 & rhs.0)
    }
}