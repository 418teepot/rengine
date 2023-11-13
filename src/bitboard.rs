use std::ops::{BitOrAssign, BitOr, BitXorAssign, BitAnd, Not, Shl, Shr, BitXor, BitAndAssign};

#[derive(PartialEq, Default, Copy, Clone, Debug)]
pub struct Bitboard(pub u64);

pub type Square = usize;

pub const NUM_OF_SQUARES: usize = 64;

impl Bitboard {
    #[allow(dead_code)]
    pub fn from_squares(squares: &[Square]) -> Self {
        let mut bitboard = Bitboard(0);
        for square in squares {
            bitboard |= Bitboard::square(*square);
        }
        bitboard
    }

    #[inline(always)]
    pub fn square(square: Square) -> Self {
        Bitboard(1 << square)
    }

    #[inline(always)]
    pub fn empty() -> Self {
        Bitboard(0)
    }

    pub fn full() -> Self {
        Bitboard(0xFFFFFFFFFFFFFFFF)
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

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    #[inline(always)]
    pub fn is_filled(&self) -> bool {
        self.0 != 0
    }

    pub fn has(&self, square: Square) -> bool {
        self.0 & (1 << square) != 0
    }

    #[inline(always)]
    pub fn next_piece_index(&self) -> Square {
        self.0.trailing_zeros() as Square
    }

    #[allow(dead_code)]
    pub fn print(self) {
        for rank in (0..8).rev() {
            for file in 0..8 {
                let square_mask = Bitboard::square(rank * 8 + file);
                if (square_mask & self).is_filled() {
                    print!("1");
                } else {
                    print!(".");
                }
            }
            println!();
        }
        println!();
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

impl BitAndAssign for Bitboard {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 = self.0 & rhs.0;
    }
}

pub struct BitboardIterator {
    bitset: Bitboard,
}

impl Iterator for BitboardIterator {
    type Item = Square;

    #[inline(always)]
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
    
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        BitboardIterator {
            bitset: self,
        }
    }
}

impl BitAnd for Bitboard {
    type Output = Bitboard;
    
    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 & rhs.0)
    }
}

impl Not for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn not(self) -> Self::Output {
        Bitboard(!self.0)
    }
}

impl Shl<usize> for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn shl(self, rhs: usize) -> Self::Output {
        Bitboard(self.0 << rhs)
    }
}

impl Shr<usize> for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn shr(self, rhs: usize) -> Self::Output {
        Bitboard(self.0 >> rhs)
    }
}

impl BitXor for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 ^ rhs.0)
    }
}