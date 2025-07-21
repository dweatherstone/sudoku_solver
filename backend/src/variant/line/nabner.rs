use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{SudokuVariant, file_parser::parse_positions, variant::Variant};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Nabner {
    cells: Vec<(usize, usize)>,
}

impl Nabner {
    pub fn new(cells: Vec<(usize, usize)>) -> Self {
        Nabner { cells }
    }

    pub fn parse(data: &str) -> Option<SudokuVariant> {
        let positions = parse_positions(data).ok()?;
        Some(SudokuVariant::Nabner(Nabner::new(positions)))
    }
}

impl Variant for Nabner {
    fn is_valid(&self, grid: &crate::SudokuGrid, row: usize, col: usize, value: u8) -> bool {
        // If the proposed cell is not in the Nabmer's cells, then return true
        if !self.cells.contains(&(row, col)) {
            return true;
        }
        let mut filled_cells = self
            .cells
            .iter()
            .filter(|&&(r, c)| !(r == row && c == col))
            .map(|&(r, c)| grid.get_cell(r, c))
            .filter(|&val| val != 0)
            .collect::<Vec<u8>>();

        // If the line already contains the value, then invalid
        if filled_cells.contains(&value) {
            return false;
        }

        // If current cell is empty, then there is nothing to constrain, so return early
        if filled_cells.is_empty() {
            return true;
        }

        // Add the proposed value to the current cells
        filled_cells.push(value);

        // If the line would be complete, use the validate_solution logic
        if filled_cells.len() == self.cells.len() {
            let mut proposed_grid = grid.clone();
            proposed_grid.set_cell(row, col, value);
            return self.validate_solution(&proposed_grid);
        }

        // Sort the values of filled_cells
        filled_cells.sort_unstable();

        // Check that each difference is at least 2
        filled_cells
            .windows(2)
            .all(|vals| vals[1].abs_diff(vals[0]) >= 2)
    }

    fn validate_solution(&self, grid: &crate::SudokuGrid) -> bool {
        let mut current_cells = self
            .cells
            .iter()
            .map(|&(r, c)| grid.get_cell(r, c))
            .filter(|&v| v != 0)
            .collect::<Vec<u8>>();

        if current_cells.len() != self.cells.len() {
            return false;
        }

        current_cells.sort_unstable();

        current_cells
            .windows(2)
            .all(|vals| vals[1].abs_diff(vals[0]) >= 2)
    }

    fn constrained_cells(&self) -> Vec<(usize, usize)> {
        self.cells.clone()
    }

    fn get_possibilities(
        &self,
        grid: &crate::SudokuGrid,
        row: usize,
        col: usize,
    ) -> HashMap<(usize, usize), Vec<u8>> {
        if !self.cells.contains(&(row, col)) {
            return HashMap::new();
        }

        let known_values: HashSet<u8> = self
            .cells
            .iter()
            .filter_map(|&(r, c)| {
                let val = grid.get_cell(r, c);
                if val == 0 { None } else { Some(val) }
            })
            .collect();
        let line_len = self.cells.len();

        // Helper function to generate all combinations of digits (1..=9) of length `line_len`
        fn gen_combinations(
            digits: &[u8],
            k: usize,
            start: usize,
            current: &mut Vec<u8>,
            result: &mut Vec<HashSet<u8>>,
            required: &HashSet<u8>,
        ) {
            if current.len() == k {
                let set: HashSet<u8> = current.iter().copied().collect();

                // Must include al known values
                if !required.is_subset(&set) {
                    return;
                }

                // No two values (in full set) can be consecutive
                let mut sorted = current.clone();
                sorted.sort_unstable();
                if sorted.windows(2).any(|w| w[1] == w[0] + 1) {
                    return;
                }

                result.push(set);
                return;
            }

            for i in start..digits.len() {
                current.push(digits[i]);
                gen_combinations(digits, k, i + 1, current, result, required);
                current.pop();
            }
        }

        let mut valid_sets = Vec::new();
        let digits: Vec<u8> = (1..=9).collect();
        gen_combinations(
            &digits,
            line_len,
            0,
            &mut Vec::new(),
            &mut valid_sets,
            &known_values,
        );

        // Determine which unplaced values are still allowed
        let mut allowed_values = HashSet::new();
        for set in &valid_sets {
            for &v in set {
                if !known_values.contains(&v) {
                    allowed_values.insert(v);
                }
            }
        }

        // Assign allowed values to unfilled cells
        let mut possibilities = HashMap::new();
        for &(r, c) in &self.cells {
            if grid.get_cell(r, c) != 0 {
                continue;
            }
            let mut values: Vec<u8> = allowed_values.iter().copied().collect();
            values.sort_unstable();
            possibilities.insert((r, c), values);
        }

        possibilities
    }
}

impl std::fmt::Display for Nabner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cells_str = self
            .cells
            .iter()
            .map(|&(r, c)| format!("({r}, {c})"))
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "Nabner Line [{cells_str}]")
    }
}

#[cfg(test)]
mod tests {
    use crate::{SudokuGrid, variant::Variant};

    use super::Nabner;

    mod is_valid {
        use super::*;

        #[test]
        fn test_valid_proposal() {
            let nabner = Nabner::new(vec![(0, 0), (0, 1), (0, 2)]);
            let mut grid = SudokuGrid::empty();
            grid.set_cell(0, 1, 4);
            grid.set_cell(0, 2, 6);
            assert!(nabner.is_valid(&grid, 0, 0, 8));
            assert!(nabner.is_valid(&grid, 0, 0, 9));
            assert!(nabner.is_valid(&grid, 0, 0, 1));
            assert!(nabner.is_valid(&grid, 0, 0, 2));
        }

        #[test]
        fn test_invalid_proposal_duplicate() {
            let nabner = Nabner::new(vec![(0, 0), (0, 1), (0, 2)]);
            let mut grid = SudokuGrid::empty();
            grid.set_cell(0, 1, 4);
            grid.set_cell(0, 2, 6);
            assert!(!nabner.is_valid(&grid, 0, 0, 4));
            assert!(!nabner.is_valid(&grid, 0, 0, 6));
        }

        #[test]
        fn test_invalid_proposal_sequence() {
            let nabner = Nabner::new(vec![(0, 0), (0, 1), (0, 2)]);
            let mut grid = SudokuGrid::empty();
            grid.set_cell(0, 1, 4);
            grid.set_cell(0, 2, 6);
            assert!(!nabner.is_valid(&grid, 0, 0, 3));
            assert!(!nabner.is_valid(&grid, 0, 0, 5));
            assert!(!nabner.is_valid(&grid, 0, 0, 7));
        }

        #[test]
        fn test_valid_proposal_incomplete() {
            let nabner = Nabner::new(vec![(0, 0), (0, 1), (0, 2), (0, 3)]);
            let mut grid = SudokuGrid::empty();
            grid.set_cell(0, 1, 4);
            grid.set_cell(0, 2, 6);
            assert!(nabner.is_valid(&grid, 0, 0, 8), "Should be valid proposal");
            assert!(nabner.is_valid(&grid, 0, 0, 9), "Should be valid proposal");
            assert!(nabner.is_valid(&grid, 0, 0, 1), "Should be valid proposal");
            assert!(nabner.is_valid(&grid, 0, 0, 2), "Should be valid proposal");
            assert!(nabner.is_valid(&grid, 0, 3, 8), "Should be valid proposal");
            assert!(nabner.is_valid(&grid, 0, 3, 9), "Should be valid proposal");
            assert!(nabner.is_valid(&grid, 0, 3, 1), "Should be valid proposal");
            assert!(nabner.is_valid(&grid, 0, 3, 2), "Should be valid proposal");
        }

        #[test]
        fn test_invalid_proposal_incomplete() {
            let nabner = Nabner::new(vec![(0, 0), (0, 1), (0, 2), (0, 3)]);
            let mut grid = SudokuGrid::empty();
            grid.set_cell(0, 1, 4);
            grid.set_cell(0, 2, 6);
            for val in 3..=7 {
                assert!(
                    !nabner.is_valid(&grid, 0, 0, val),
                    "Should be invalid proposal"
                );
                assert!(
                    !nabner.is_valid(&grid, 0, 3, val),
                    "Should be invalid proposal"
                );
            }
        }

        #[test]
        fn test_valid_not_on_line() {
            let nabner = Nabner::new(vec![(0, 0), (0, 1), (0, 2)]);
            let grid = SudokuGrid::empty();
            assert!(nabner.is_valid(&grid, 1, 1, 5));
        }
    }

    mod validate_solution {
        use super::*;

        #[test]
        fn test_single_cell_valid() {
            let nabner = Nabner::new(vec![(4, 4)]);
            let mut grid = SudokuGrid::empty();
            grid.set_cell(4, 4, 5);
            assert!(nabner.validate_solution(&grid));
        }

        #[test]
        fn test_max_length_line_success() {
            let nabner = Nabner::new(vec![(0, 0), (0, 1), (0, 2), (0, 3), (0, 4)]);
            let mut grid = SudokuGrid::empty();
            grid.set_cell(0, 0, 5);
            grid.set_cell(0, 1, 9);
            grid.set_cell(0, 2, 3);
            grid.set_cell(0, 3, 1);
            grid.set_cell(0, 4, 7);
            assert!(nabner.validate_solution(&grid));

            let nabner = Nabner::new(vec![(0, 0), (0, 1), (0, 2), (0, 3)]);
            grid = SudokuGrid::empty();
            grid.set_cell(0, 0, 4);
            grid.set_cell(0, 1, 2);
            grid.set_cell(0, 2, 6);
            grid.set_cell(0, 3, 8);
            assert!(nabner.validate_solution(&grid));
        }

        #[test]
        fn test_line_incomplete() {
            let nabner = Nabner::new(vec![(0, 0), (0, 1)]);
            let mut grid = SudokuGrid::empty();
            grid.set_cell(0, 1, 3);
            assert!(!nabner.validate_solution(&grid));
        }

        #[test]
        fn test_line_with_duplicates() {
            let nabner = Nabner::new(vec![(0, 0), (0, 1)]);
            let mut grid = SudokuGrid::empty();
            grid.set_cell(0, 1, 3);
            grid.set_cell(0, 0, 3);
            assert!(!nabner.validate_solution(&grid));
        }

        #[test]
        fn test_line_with_consecutive() {
            let nabner = Nabner::new(vec![(0, 0), (0, 1)]);
            let mut grid = SudokuGrid::empty();
            grid.set_cell(0, 1, 3);
            grid.set_cell(0, 0, 4);
            assert!(!nabner.validate_solution(&grid));
            grid.set_cell(0, 0, 2);
            assert!(!nabner.validate_solution(&grid));
        }
    }

    mod get_possibilities {
        use super::*;

        #[test]
        fn test_single_option_remaining() {
            let nabner = Nabner::new(vec![(0, 0), (0, 1), (0, 2), (0, 3)]);
            let mut grid = SudokuGrid::empty();
            grid.set_cell(0, 0, 4);
            grid.set_cell(0, 1, 2);
            grid.set_cell(0, 2, 7);
            let result = nabner.get_possibilities(&grid, 0, 2);
            assert_eq!(result.len(), 1);
            assert_eq!(result.get(&(0, 3)), Some(&vec![9]));
        }

        #[test]
        fn test_many_possibilities() {
            let nabner = Nabner::new(vec![(0, 0), (0, 1), (0, 2), (0, 3)]);
            let mut grid = SudokuGrid::empty();
            grid.set_cell(0, 0, 4);
            let expected = vec![1, 2, 6, 7, 8, 9];
            let result = nabner.get_possibilities(&grid, 0, 0);
            assert_eq!(result.len(), 3);
            for cell in [(0, 1), (0, 2), (0, 3)] {
                assert_eq!(result.get(&cell).unwrap(), &expected);
            }
        }
    }
}
