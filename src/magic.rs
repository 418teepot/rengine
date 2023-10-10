use crate::bitboard::{Bitboard, Square};
use crate::gamestate::{Piece, ROOK, BISHOP};
use rand::{thread_rng, Rng};

pub static mailbox: [i8; 120] = [
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1,  0,  1,  2,  3,  4,  5,  6,  7, -1,
    -1,  8,  9, 10, 11, 12, 13, 14, 15, -1,
    -1, 16, 17, 18, 19, 20, 21, 22, 23, -1,
    -1, 24, 25, 26, 27, 28, 29, 30, 31, -1,
    -1, 32, 33, 34, 35, 36, 37, 38, 39, -1,
    -1, 40, 41, 42, 43, 44, 45, 46, 47, -1,
    -1, 48, 49, 50, 51, 52, 53, 54, 55, -1,
    -1, 56, 57, 58, 59, 60, 61, 62, 63, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1
];

pub static mailbox64: [i8; 64] = [
    21, 22, 23, 24, 25, 26, 27, 28,
    31, 32, 33, 34, 35, 36, 37, 38,
    41, 42, 43, 44, 45, 46, 47, 48,
    51, 52, 53, 54, 55, 56, 57, 58,
    61, 62, 63, 64, 65, 66, 67, 68,
    71, 72, 73, 74, 75, 76, 77, 78,
    81, 82, 83, 84, 85, 86, 87, 88,
    91, 92, 93, 94, 95, 96, 97, 98
];

#[derive(Clone, Copy, Default, Debug)]
pub struct MagicEntry {
    mask: Bitboard,
    magic: u64,
    index_bits: u8,
}

#[inline(always)]
pub fn magic_index(entry: MagicEntry, blockers: Bitboard) -> usize {
    let relevant_blockers = blockers & entry.mask;
    let hash = relevant_blockers.0.wrapping_mul(entry.magic);
    (hash >> (64 - entry.index_bits)) as usize
}

fn find_magic_entry(piece: Piece, square: Square) -> (MagicEntry, Vec<Bitboard>) {
    let relevant_mask = relevant_slider_blockers(square, piece);
    let mut rng = rand::thread_rng();
    loop {
        let magic_candidate = rng.gen::<u64>() & rng.gen::<u64>();
        let magic_entry_candidate = MagicEntry { mask: relevant_mask, magic: magic_candidate, index_bits: (relevant_mask.0.count_ones() + 3) as u8 };
        if let Some(table) = try_make_table(piece, square, magic_entry_candidate) {
            return (magic_entry_candidate, table);
        }
    }
}

fn try_make_table(piece: Piece, square: Square, entry_candidate: MagicEntry) -> Option<Vec<Bitboard>> {
    let mut possible_table = vec![Bitboard::empty(); 1 << entry_candidate.index_bits];
    let relevancy_mask = relevant_slider_blockers(square, piece);
    for blockers in relevancy_mask.iter_all_subsets() {
        let plays = slider_plays_for_blockers(square, piece, blockers);
        let magic_index = magic_index(entry_candidate, blockers);
        if possible_table[magic_index] == Bitboard(0) {
            possible_table[magic_index] = plays;
        } else {
            return None;
        }
    }
    Some(possible_table)
}

pub fn relevant_slider_blockers(square: Square, piece: Piece) -> Bitboard {
    let mut blockers = 0;
    let rays = get_rays_for_piece(piece);
    for ray in rays {
        let mut i = 1;
        if mailbox[(mailbox64[square] + ray * i) as usize] != -1 {
            loop {
                if mailbox[(mailbox64[square] + ray * (i+1)) as usize] == -1 {
                    break;
                }
                blockers |= 1 << mailbox[(mailbox64[square] + ray * i) as usize];
                i += 1;
            }
        }
    }
    Bitboard(blockers)
}

pub fn slider_plays_for_blockers(square: Square, piece: Piece, blockers: Bitboard) -> Bitboard {
    let mut plays = Bitboard(0);
    let rays = get_rays_for_piece(piece);
    for ray in rays {
        let mut i = 1;
        loop {
            if mailbox[(mailbox64[square] + ray * i) as usize] == -1 {
                break;
            }

            plays |= Bitboard::square(mailbox[(mailbox64[square] + ray * i) as usize] as Square);

            if Bitboard::square(mailbox[(mailbox64[square] + ray * i) as usize] as usize) & blockers != Bitboard(0) {
                break;
            }

            i += 1;
        }
    }
    plays
}

static ROOK_RAYS: [i8; 4] = [1, -1, 10, -10];
static BISHOP_RAYS: [i8; 4] = [11, -11, -9, 9];

fn get_rays_for_piece(piece: Piece) -> &'static [i8] {
    if piece == ROOK { &ROOK_RAYS } else if piece == BISHOP { &BISHOP_RAYS } else { panic!("Wrong piece in blokcer mask inputed.") }
}

pub struct BitSubset {
    set: Bitboard,
    subset: Bitboard,
}

impl BitSubset {
    pub fn new(set: Bitboard) -> BitSubset {
        BitSubset {
            set,
            subset: Bitboard(0),
        }
    }
}

impl Iterator for BitSubset {
    type Item = Bitboard;

    fn next(&mut self) -> Option<Self::Item> {
        self.subset = Bitboard(self.subset.0.wrapping_sub(self.set.0)) & self.set;
        if self.subset == Bitboard(0) {
            None
        } else {
            Some(self.subset)
        }
    }
}

impl Bitboard {
    fn iter_all_subsets(&self) -> BitSubset {
        BitSubset::new(*self)
    }
}

lazy_static! {
    pub static ref ROOK_MAGICS_AND_PLAYS: Vec<(MagicEntry, Vec<Bitboard>)> = {
        let mut magics_and_plays = vec![];
        for square in 0..64 {
            magics_and_plays.push(find_magic_entry(ROOK, square));
        }

        magics_and_plays
    };

    pub static ref BISHOP_MAGICS_AND_PLAYS: Vec<(MagicEntry, Vec<Bitboard>)> = {
        let mut magics_and_plays = vec![];
        for square in 0..64 {
            magics_and_plays.push(find_magic_entry(BISHOP, square));
        }
        magics_and_plays
    };
}