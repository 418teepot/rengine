use std::collections::HashMap;
use std::fs::read_to_string;

lazy_static! {
    pub static ref OPENING_BOOK: HashMap<String, Vec<(String, u32)>> = {
        let mut book = HashMap::new();

        let book_file_contents = read_to_string("resources/book.txt").unwrap_or("".to_string());

        let mut line_iterator = book_file_contents.lines();
        
        let mut pos_string: String = String::new();
        let mut moves = Vec::new();
        while let Some(line) = line_iterator.next() {
            if line.starts_with("pos") {
                book.insert(pos_string.to_string(), moves);
                pos_string = line[4..].to_string();
                moves = Vec::new();
                continue;
            }
            let line_split: Vec<&str> = line.split_whitespace().collect();
            let r#move = line_split[0];
            let frequency: u32 = line_split[1].parse().unwrap();
            moves.push((r#move.to_string(), frequency));
        }
        book
    };
}