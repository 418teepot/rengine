use std::ops::{BitOrAssign, BitOr, BitXorAssign};

#[derive(PartialEq, Default, Copy, Clone)]
pub struct Bitboard(i64);

pub type Square = usize;

impl Bitboard {
    pub fn from_squares(squares: &[Square]) -> Self {
        let mut bitboard = Bitboard(0);
        for square in squares {
            bitboard |= Bitboard(1 << square);
        }
        bitboard
    }

    #[inline(always)]
    pub fn square(square: Square) -> Self {
        Bitboard(1 << square)
    }

    pub fn empty() -> Self {
        Bitboard(0)
    }

    #[inline(always)]
    pub fn add_piece(&mut self, square: Square) {
        self.0 |= 1 << square;
    } 

    // TODO: Secure xor wihtout having to check if square is actually set?
    #[inline(always)]
    pub fn remove_piece(&mut self, square: Square) {
        self.0 &= !(1 << square);
    }
}

impl BitOr for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Bitboard {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 = self.0 | rhs.0;
    }
}

impl BitXorAssign for Bitboard {
    #[inline(always)]
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
        let index = self.bitset.0.trailing_zeros() as usize;
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