#![feature(link_args)]

/// A problem-solving strategy for the n-queens problem.
pub trait NQueensStrategy: Sized {
    /// Extra parameters that may be given to the challenge to configure the
    /// solution.
    type Config;

    /// Creates a new solvable instance of this challenge.
    fn new(dimension: usize, config: Self::Config) -> Self;

    /// Solves the challenge for returning a vector with `n` positions,
    /// indicated as a vector into an array of dimension * dimension.
    ///
    /// Each step will call step_callback, if appropriate, with the current
    /// state of the board.
    fn solve<F>(self, step_callback: F) -> Option<Vec<(usize, usize)>>
        where F: FnMut(&[(usize, usize)]);
}

pub mod hill_climbing {
    use super::*;

    /// A hill-climbing solution to the n-queens challenge
    pub struct HillClimbing {
        size: usize,
        positions: Vec<(usize, usize)>,
    }

    enum PositionError {
        /// A queen is already there.
        Match,
        /// Queen in the same column.
        Column,
        /// Queen in the same row.
        Row,
        /// Queen in the same diagonal.
        Diagonal,
    }

    impl HillClimbing {
        fn dimension(&self) -> usize {
            self.size
        }

        /// Returns true if a queen positioned at `one` could be hit by a queen
        /// positioned at `other`.
        fn can_position(&self,
                        (x1, y1): (usize, usize),
                        (x2, y2): (usize, usize))
                        -> Result<(), PositionError> {
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
                                      -> Result<(), PositionError> {
            for existing_position in &self.positions {
                if let Err(err) = self.can_position(pos, *existing_position) {
                    return Err(err);
                }
            }

            Ok(())
        }

        /// Tries to position the next queen in a greedy way. Gets an immutable
        /// view of the current positions, and returns the position the next
        /// queen is at, or an error if it can't be positioned.
        fn position_next_queen_starting_from(&self,
                                             (mut x, mut y): (usize, usize))
                                             -> Result<(usize, usize), ()> {
            while x < self.size && y < self.size {
                match self.queen_can_be_positioned_at((x, y)) {
                    Ok(()) => return Ok((x, y)),
                    Err(PositionError::Match) => {
                        x += 1;
                        y = 0;
                    }
                    Err(PositionError::Column) => {
                        x += 1;
                    }
                    Err(PositionError::Diagonal) |
                    Err(PositionError::Row) => {
                        if y == self.size - 1 {
                            y = 0;
                            x += 1;
                        } else {
                            y += 1;
                        }
                    }
                }
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
                positions: Vec::with_capacity(dimensions),
            }
        }

        fn solve<F>(mut self, mut step_callback: F) -> Option<Vec<(usize, usize)>>
            where F: FnMut(&[(usize, usize)]),
        {
            if self.dimension() == 0 {
                return Some(vec![]);
            }

            let mut start_search_at = (0, 0);
            while self.positions.len() != self.dimension() {
                match self.position_next_queen_starting_from(start_search_at) {
                    Ok(pos) => {
                        self.positions.push(pos);
                        step_callback(&self.positions);
                        start_search_at = (0, 0);
                    }
                    Err(()) => {
                        match self.positions.pop() {
                            Some((x, y)) => {
                                start_search_at =
                                    if y == self.size - 1 {
                                        (x + 1, 0)
                                    } else {
                                        (x, y + 1)
                                    };
                            }
                            // No solution.
                            None => return None,
                        }
                    }
                }
            }

            return Some(self.positions);
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
            assert!(challenge.solve(|_| {}).is_some());
        }

        #[test]
        fn finds_twelve_queens_solution() {
            let challenge = HillClimbing::new(12, ());
            assert!(challenge.solve(|_| {}).is_some());
        }

        #[test]
        fn finds_fifteen_queens_solution() {
            let challenge = HillClimbing::new(15, ());
            assert!(challenge.solve(|_| {}).is_some());
        }
    }
}

pub type JSCallback = extern "C" fn(positions: *const usize, len: usize);

pub fn solve<T: NQueensStrategy>(n: usize,
                                 result_storage: *mut usize,
                                 _callback: Option<JSCallback>,
                                 config: T::Config)
                                 -> usize {
    use std::slice;

    let challenge = T::new(n, config);
    let result = challenge.solve(|_step| {
        // FIXME(emilio): Invoke callback, but I don't need it right now.
        // if let Some(cb) = callback {
        //     cb(step.as_ptr(), step.len());
        // }
    });

    let result = match result {
        Some(result) => result,
        None => return 0,
    };

    let mut storage = unsafe { slice::from_raw_parts_mut(result_storage, n) };
    for (i, (x, y)) in result.into_iter().enumerate() {
        storage[i] = x + y * n;
    }

    return 1;
}

#[cfg(not(test))]
#[link_args = "-s EXPORTED_FUNCTIONS=['_solve_n_queens_hill_climbing'] -s RESERVED_FUNCTION_POINTERS=20"]
extern {}

#[no_mangle]
pub fn solve_n_queens_hill_climbing(n: usize,
                                    result_storage: *mut usize,
                                    cb: Option<JSCallback>)
                                    -> usize {
    solve::<hill_climbing::HillClimbing>(n, result_storage, cb, ())
}

fn main() { /* Intentionally empty */ }
