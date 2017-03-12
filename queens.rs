#![feature(link_args)]

pub enum PositionError {
    /// A queen is already there.
    Match,
    /// Queen in the same column.
    Column,
    /// Queen in the same row.
    Row,
    /// Queen in the same diagonal.
    Diagonal,
}

/// A problem-solving strategy for the n-queens problem.
pub trait NQueensStrategy: Sized {
    /// Extra parameters that may be given to the challenge to configure the
    /// solution.
    type Config;

    /// Creates a new solvable instance of this challenge.
    fn new(dimension: usize, config: Self::Config) -> Self;

    /// Solves the challenge for returning a vector with `n` positions,
    /// representing the column at which the queen is positioned for each index.
    fn solve(self) -> Option<Box<[usize]>> {
        self.solve_with_callback(|_| {})
    }

    /// Solves the challenge for returning a vector with `n` positions,
    /// representing the column at which the queen is positioned for each index,
    /// and additionally runs `callback` on each step the potisions changed.
    fn solve_with_callback<F>(self, callback: F) -> Option<Box<[usize]>>
        where F: FnMut(&[usize]);

    /// Get the currently positioned queen rows.
    fn queen_rows(&self) -> &[usize];

    /// Returns true if a queen positioned at `one` could be hit by a queen
    /// positioned at `other`.
    fn can_position(&self,
                    p1: (usize, usize),
                    p2: (usize, usize))
                    -> Result<(), PositionError> {
        let (x1, y1) = p1;
        let (x2, y2) = p2;

        if x1 == x2 && y1 == y2 {
            return Err(PositionError::Match);
        }

        if x1 == x2 {
            return Err(PositionError::Column);
        }

        if y1 == y2 {
            return Err(PositionError::Row);
        }

        let x_difference = (x1 as isize - x2 as isize).abs();
        let y_difference = (y1 as isize - y2 as isize).abs();

        if x_difference == y_difference {
            return Err(PositionError::Diagonal);
        }

        Ok(())
    }

    fn queen_can_be_positioned_at(&self,
                                  pos: (usize, usize))
                                  -> bool {
        for (x, &y) in self.queen_rows().iter().enumerate() {
            if self.can_position(pos, (x, y)).is_err() {
                return false;
            }
        }

        true
    }
}

pub mod hill_climbing {
    use super::*;

    /// A hill-climbing solution to the n-queens challenge
    pub struct HillClimbing {
        size: usize,
        /// NOTE: We already know the row they are, because all the queens must
        /// necessarily be in a different one.
        queen_rows: Vec<usize>,
    }

    impl HillClimbing {
        fn dimension(&self) -> usize {
            self.size
        }

        /// Tries to position the next queen at row `row`, or any of the
        /// following columns.
        fn position_next_queen_from_row(&self, mut row: usize)
                                        -> Result<usize, ()> {
            while row < self.size {
                if self.queen_can_be_positioned_at((self.queen_rows.len(), row)) {
                    return Ok(row);
                }
                row += 1;
            }

            Err(())
        }
    }

    impl NQueensStrategy for HillClimbing {
        /// No configuration needed.
        type Config = ();

        fn new(dimensions: usize, _: ()) -> Self {
            HillClimbing {
                size: dimensions,
                queen_rows: Vec::with_capacity(dimensions),
            }
        }

        fn queen_rows(&self) -> &[usize] {
            &self.queen_rows
        }

        fn solve_with_callback<F>(mut self,
                                  mut callback: F)
                                  -> Option<Box<[usize]>>
            where F: FnMut(&[usize]),
        {
            if self.size == 0 {
                return Some(vec![].into_boxed_slice());
            }

            let mut start_search_at = 0;
            while self.queen_rows.len() != self.size {
                match self.position_next_queen_from_row(start_search_at) {
                    Ok(pos) => {
                        self.queen_rows.push(pos);
                        callback(&self.queen_rows);
                        start_search_at = 0;
                    }
                    Err(()) => {
                        match self.queen_rows.pop() {
                            Some(row) => {
                                callback(&self.queen_rows);
                                start_search_at = row + 1;
                            }
                            // No solution.
                            None => return None,
                        }
                    }
                }
            }

            return Some(self.queen_rows.into_boxed_slice());
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        const DIM: usize = 8;
        fn pos(x: usize, y: usize) -> (usize, usize) {
            (x, y)
        }

        #[test]
        fn are_reachable_test() {
            let challenge = HillClimbing::new(DIM, ());

            assert!(challenge.can_position(pos(0, 0), pos(0, 0)).is_err());
            assert!(challenge.can_position(pos(0, 1), pos(0, 0)).is_err());
            assert!(challenge.can_position(pos(1, 0), pos(0, 0)).is_err());
            assert!(challenge.can_position(pos(1, 1), pos(5, 5)).is_err());
            assert!(challenge.can_position(pos(3, 2), pos(2, 3)).is_err());
        }

        #[test]
        fn finds_eight_queens_solution() {
            let challenge = HillClimbing::new(DIM, ());
            assert!(challenge.solve().is_some());
        }

        #[test]
        fn finds_twelve_queens_solution() {
            let challenge = HillClimbing::new(12, ());
            assert!(challenge.solve().is_some());
        }

        #[test]
        fn finds_fifteen_queens_solution() {
            let challenge = HillClimbing::new(15, ());
            assert!(challenge.solve().is_some());
        }
    }
}

pub mod simulated_annealing {
    use super::*;

    pub struct SimulatedAnnealingConfig {
        starting_temperature: usize,
        cooling_factor: f32,
    }

    pub struct SimulatedAnnealing {
        temperature: usize,
        cooling_factor: f32,
        size: usize,
        queen_rows: Vec<usize>,
    }

    impl NQueensStrategy for SimulatedAnnealing {
        type Config = SimulatedAnnealingConfig;

        fn new(dimensions: usize, config: Self::Config) -> Self {
            SimulatedAnnealing {
                temperature: config.starting_temperature,
                cooling_factor: config.cooling_factor,
                size: dimensions,
                queen_rows: Vec::with_capacity(dimensions),
            }
        }

        fn queen_rows(&self) -> &[usize] {
            &self.queen_rows
        }

        fn solve_with_callback<F>(mut self,
                                  mut callback: F)
                                  -> Option<Box<[usize]>>
            where F: FnMut(&[usize]),
        {
            if self.size == 0 {
                return Some(vec![].into_boxed_slice());
            }

            unimplemented!();
        }
    }
}

pub fn solve<T: NQueensStrategy>(n: usize,
                                 result_storage: *mut usize,
                                 _callback: Option<JSCallback>,
                                 config: T::Config)
                                 -> usize {
    use std::slice;

    let challenge = T::new(n, config);
    let result = challenge.solve();

    let result = match result {
        Some(result) => result,
        None => return 0,
    };

    let mut storage = unsafe { slice::from_raw_parts_mut(result_storage, n) };
    for (x, y) in result.into_iter().enumerate() {
        storage[x] = x + y * n;
    }

    return 1;
}

#[cfg(not(test))]
#[link_args = "-s EXPORTED_FUNCTIONS=['_solve_n_queens_hill_climbing'] -s RESERVED_FUNCTION_POINTERS=20"]
extern {}

// TODO(emilio): Get rid of this.
pub type JSCallback = extern "C" fn(positions: *const usize, len: usize);

#[no_mangle]
pub fn solve_n_queens_hill_climbing(n: usize,
                                    result_storage: *mut usize,
                                    cb: Option<JSCallback>)
                                    -> usize {
    solve::<hill_climbing::HillClimbing>(n, result_storage, cb, ())
}

fn main() { /* Intentionally empty */ }
