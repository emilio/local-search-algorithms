#![feature(link_args)]

extern crate rand;

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

pub struct Solution {
    queen_rows: Box<[usize]>,
    score: usize,
}

impl Solution {
    pub fn new(queen_rows: Vec<usize>, score: usize) -> Self {
        Solution {
            queen_rows: queen_rows.into_boxed_slice(),
            score: score,
        }
    }
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
    fn solve(self) -> Solution {
        self.solve_with_callback(|_, _| {})
    }

    /// Solves the challenge for returning a vector with `n` positions,
    /// representing the column at which the queen is positioned for each index,
    /// and additionally runs `callback` on each step the positions changed,
    /// with the queen positions and the current score so far.
    fn solve_with_callback<F>(self, callback: F) -> Solution
        where F: FnMut(&[usize], usize);

    /// Get the currently positioned queen rows.
    fn queen_rows(&self) -> &[usize];

    /// Get the size of the board.
    fn size(&self) -> usize;

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

    fn can_hit(&self,
               p1: (usize, usize),
               p2: (usize, usize)) -> bool {
        self.can_position(p1, p2).is_err()
    }

    /// Returns true if the current algorithm can't find a better solution,
    /// because all the queens can't hit each other.
    fn done(&self) -> bool {
        let size = self.size();
        let rows = self.queen_rows();

        // Still unpositioned queens.
        if size != rows.len() {
            return false;
        }

        for i in 0..size {
            for j in (i + 1)..size {
                if self.can_hit((i, rows[i]), (j, rows[j])) {
                    return false;
                }
            }
        }

        debug_assert!(self.score() == 0);
        return true;
    }

    /// Returns the number of pairs of queens that can hit each other.
    fn score(&self) -> usize {
        let rows = self.queen_rows();

        let mut score = 0;

        for i in 0..rows.len() {
            for j in (i + 1)..rows.len() {
                if self.can_hit((i, rows[i]), (j, rows[j])) {
                    score += 1;
                }
            }
        }

        score
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

pub mod constraint_propagation {
    use super::*;

    /// A constraint-propagation solution to the n-queens challenge.
    pub struct ConstraintPropagation {
        size: usize,
        /// NOTE: We already know the row they are, because all the queens must
        /// necessarily be in a different one.
        queen_rows: Vec<usize>,
    }

    impl ConstraintPropagation {
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

    impl NQueensStrategy for ConstraintPropagation {
        /// No configuration needed.
        type Config = ();

        fn new(dimensions: usize, _: ()) -> Self {
            ConstraintPropagation {
                size: dimensions,
                queen_rows: Vec::with_capacity(dimensions),
            }
        }

        fn queen_rows(&self) -> &[usize] { &self.queen_rows }
        fn size(&self) -> usize { self.size }

        fn solve_with_callback<F>(mut self, mut callback: F) -> Solution
            where F: FnMut(&[usize], usize),
        {
            let mut start_search_at = 0;
            while self.queen_rows.len() != self.size {
                match self.position_next_queen_from_row(start_search_at) {
                    Ok(pos) => {
                        self.queen_rows.push(pos);
                        callback(&self.queen_rows, 0);
                        start_search_at = 0;
                    }
                    Err(()) => {
                        match self.queen_rows.pop() {
                            Some(row) => {
                                callback(&self.queen_rows, 0);
                                start_search_at = row + 1;
                            }
                            // Not a single solution.
                            None => break,
                        }
                    }
                }
            }

            let score = self.score();
            Solution::new(self.queen_rows, score)
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
            let challenge = ConstraintPropagation::new(DIM, ());

            assert!(challenge.can_position(pos(0, 0), pos(0, 0)).is_err());
            assert!(challenge.can_position(pos(0, 1), pos(0, 0)).is_err());
            assert!(challenge.can_position(pos(1, 0), pos(0, 0)).is_err());
            assert!(challenge.can_position(pos(1, 1), pos(5, 5)).is_err());
            assert!(challenge.can_position(pos(3, 2), pos(2, 3)).is_err());
        }

        #[test]
        fn finds_eight_queens_solution() {
            let challenge = ConstraintPropagation::new(DIM, ());
            assert_eq!(challenge.solve().score, 0);
        }

        #[test]
        fn finds_twelve_queens_solution() {
            let challenge = ConstraintPropagation::new(12, ());
            assert_eq!(challenge.solve().score, 0);
        }

        #[test]
        fn finds_fifteen_queens_solution() {
            let challenge = ConstraintPropagation::new(15, ());
            assert_eq!(challenge.solve().score, 0);
        }
    }
}

pub mod hill_climbing {
    use super::*;
    use rand::Rng;

    pub struct HillClimbing {
        size: usize,
        queen_rows: Vec<usize>,
        rng: rand::OsRng,
    }

    impl HillClimbing {
        fn random_queen_index(&mut self) -> usize {
            self.rng.next_u32() as usize % self.queen_rows.len()
        }
    }

    impl NQueensStrategy for HillClimbing {
        /// No configuration needed.
        type Config = ();

        fn new(size: usize, _: ()) -> Self {
            let mut rng = rand::OsRng::new().unwrap();
            let mut positions_pending =
                (0..size).into_iter().collect::<Vec<_>>();

            let mut queen_rows = vec![0; size];

            // Distribute the initial positions randomly.
            while !positions_pending.is_empty() {
                let chosen =
                    rng.next_u32() as usize % positions_pending.len();

                let position = positions_pending.remove(chosen);
                queen_rows[positions_pending.len()] = position;
            }

            HillClimbing {
                size: size,
                queen_rows: queen_rows,
                rng: rng,
            }
        }

        fn queen_rows(&self) -> &[usize] { &self.queen_rows }
        fn size(&self) -> usize { self.size }

        fn solve_with_callback<F>(mut self, mut callback: F) -> Solution
            where F: FnMut(&[usize], usize),
        {
            use std::mem;
            const MAX_ITERATIONS_WITHOUT_IMPROVEMENT: usize = 1000;

            let mut current_score = self.score();
            let mut iterations_without_improvement = 0;

            callback(&self.queen_rows(), current_score);

            while current_score != 0 &&
                  iterations_without_improvement <= MAX_ITERATIONS_WITHOUT_IMPROVEMENT {
                let mut queen_1 = self.random_queen_index();
                let mut queen_2 = self.random_queen_index();
                while queen_1 == queen_2 {
                    queen_2 = self.random_queen_index();
                }

                // Swap them, and check score.
                self.queen_rows.swap(queen_1, queen_2);

                let score = self.score();
                if score < current_score {
                    // Yay, an improvement! Let's leave the stuff as-is :)
                    iterations_without_improvement = 0;
                    current_score = score;
                    callback(&self.queen_rows, current_score)
                } else {
                    // Didn't improve, let's just get back to where we were.
                    iterations_without_improvement += 1;
                    self.queen_rows.swap(queen_1, queen_2);
                }
            }

            Solution::new(self.queen_rows, current_score)
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

        fn queen_rows(&self) -> &[usize] { &self.queen_rows }
        fn size(&self) -> usize { self.size }

        fn solve_with_callback<F>(mut self, mut callback: F) -> Solution
            where F: FnMut(&[usize], usize),
        {
            unimplemented!();
        }
    }
}

pub fn solve<T: NQueensStrategy>(n: usize,
                                 result_storage: *mut usize,
                                 callback: Option<JSCallback>,
                                 config: T::Config)
                                 -> usize {
    use std::slice;

    let challenge = T::new(n, config);
    let solution = challenge.solve_with_callback(|queens, score| {
        if let Some(cb) = callback {
            cb(queens.as_ptr(), queens.len(), score)
        }
    });

    let mut storage = unsafe { slice::from_raw_parts_mut(result_storage, n + 1) };
    storage[0] = solution.queen_rows.len();

    // TODO(emilio): This is inconsistent with the data passed to the callback.
    for (x, y) in solution.queen_rows.into_iter().enumerate() {
        storage[x + 1] = x + y * n;
    }

    solution.score
}

#[cfg(target_os = "emscripten")]
#[link_args = "-s EXPORTED_FUNCTIONS=['_solve_n_queens_constraint_propagation','_solve_n_queens_hill_climbing'] -s RESERVED_FUNCTION_POINTERS=20"]
extern {}

pub type JSCallback = extern "C" fn(positions: *const usize,
                                    len: usize,
                                    score: usize);

#[no_mangle]
pub fn solve_n_queens_constraint_propagation(n: usize,
                                    result_storage: *mut usize,
                                    cb: Option<JSCallback>)
                                    -> usize {
    solve::<constraint_propagation::ConstraintPropagation>(n, result_storage, cb, ())
}

#[no_mangle]
pub fn solve_n_queens_hill_climbing(n: usize,
                                    result_storage: *mut usize,
                                    cb: Option<JSCallback>)
                                    -> usize {
    solve::<hill_climbing::HillClimbing>(n, result_storage, cb, ())
}

fn main() { /* Intentionally empty */ }
