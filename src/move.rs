#[derive(Default, Copy, Clone)]
pub struct Move(i32);

const MAX_MOVES: usize = 100;
const NULLMOVE: Move = Move(0);

struct MoveList {
    moves: [Move; MAX_MOVES],
    length: u8,
}

impl MoveList {
    fn new() -> MoveList {
        MoveList {
            moves: [NULLMOVE; MAX_MOVES],
            length: 0,
        }
    }

    fn add_move(&mut self, r#move: Move) {
        assert!(self.length as usize <= MAX_MOVES);
        self.moves[self.length as usize] = r#move;
        self.length += 1;
    }
}

struct MoveIterator {
    move_list: MoveList,
    index: u8,
}

impl Iterator for MoveIterator {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.move_list.length - 1 {
            return None;
        }
        let r#move = self.move_list.moves[self.index as usize];
        self.index += 1;
        Some(r#move)
    }
}

impl IntoIterator for MoveList {
    type Item = Move;

    type IntoIter = MoveIterator;

    fn into_iter(self) -> Self::IntoIter {
        MoveIterator {
            move_list: self,
            index: 0,
        }
    }
}