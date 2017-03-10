
struct ChessBoard {
    dimensions: usize,
    board: Box<[bool]>,
}

impl ChessBoard {
    fn new(dimensions: usize) -> Self {
        ChessBoard {
            dimensions: dimensions,
            board: vec![false; dimensions * dimensions].into_boxed_slice(),
        }
    }
}

fn main() {
    let chess = ChessBoard::new(8);
    println!("Have {} queens", chess.dimensions);
}
