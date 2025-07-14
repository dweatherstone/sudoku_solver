use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{SudokuVariant, file_parser::parse_positions, variant::Variant};

/// A Killer cage where a number of cells must sum to a given number, and there must be no repeated values in the cage.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct KillerCage {
    cells: Vec<(usize, usize)>,
    total: u8,
    possible_values: HashSet<u8>,
}

impl KillerCage {
    ///  Creates a Killer Cage comprising the given cells, and summing to the given value.
    pub fn new(cells: Vec<(usize, usize)>, sum: u8) -> Self {
        let mut cage = KillerCage {
            cells,
            total: sum,
            possible_values: HashSet::new(),
        };
        cage.set_possible_values();
        cage
    }

    /// Parses a string into an `Killer` `SudokuVariant`.
    /// The string is expected to be of the form:
    /// Killer: ([cells]): sum
    /// e.g. "Killer: ((0, 1), (0, 2), (1, 1)): 15"
    ///
    ///
    /// # Examples:
    /// ```
    /// use sudoku_solver::{SudokuVariant, KillerCage};
    /// let optional_variant = KillerCage::parse("((0, 1), (0, 2), (1, 1)): 15");
    /// ```
    pub fn parse(data: &str) -> Option<SudokuVariant> {
        let parts: Vec<&str> = data.split(':').collect();
        if parts.len() != 2 {
            return None;
        }
        let cells = parse_positions(parts[0].trim()).ok()?;
        let sum = parts[1].trim().parse().ok()?;
        Some(SudokuVariant::Killer(KillerCage::new(cells, sum)))
    }

    // Calculates the possible values for the given killer cage
    fn set_possible_values(&mut self) {
        let digits = (1u8..=9).collect::<Vec<_>>();
        let mut result = HashSet::new();

        // Recursive helper to generate combinations
        fn backtrack(
            digits: &[u8],
            size: usize,
            target_sum: u8,
            start: usize,
            current_combo: &mut Vec<u8>,
            result: &mut HashSet<u8>,
        ) {
            if size == 0 {
                if target_sum == 0 {
                    // Valid combo found: add all digits to result
                    for &d in current_combo.iter() {
                        result.insert(d);
                    }
                }
                return;
            }
            for i in start..digits.len() {
                let d = digits[i];
                if d > target_sum {
                    // Prune: digits are sorted ascending, no point going further
                    break;
                }
                current_combo.push(digits[i]);
                backtrack(
                    digits,
                    size - 1,
                    target_sum - d,
                    i + 1,
                    current_combo,
                    result,
                );
                current_combo.pop();
            }
        }

        backtrack(
            &digits,
            self.cells.len(),
            self.total,
            0,
            &mut Vec::new(),
            &mut result,
        );
        self.possible_values = result;
    }
}

impl Variant for KillerCage {
    fn is_valid(&self, grid: &crate::SudokuGrid, row: usize, col: usize, value: u8) -> bool {
        // If (row, col) is not in the cage, just pass
        if !self.cells.contains(&(row, col)) {
            return true;
        }

        if !self.possible_values.contains(&value) {
            return false;
        }

        // If the cage already contains the value, then invalid
        if self
            .cells
            .iter()
            .filter(|&&(r, c)| !(r == row && c == col)) // ignore current cell
            .map(|&(r, c)| grid.get_cell(r, c))
            .any(|val| val == value)
        {
            return false;
        }

        // Check that the sum of all filled values in the cage doesn't exceed the required sum
        let mut current_sum = 0;
        let mut empty_cells = 0;

        for &(r, c) in &self.cells {
            let val = grid.get_cell(r, c);
            if val == 0 && (r, c) != (row, col) {
                empty_cells += 1;
            }
            current_sum += if (r, c) == (row, col) { value } else { val };
        }

        if empty_cells == 0 {
            current_sum == self.total
        } else {
            current_sum <= self.total
        }
    }

    fn validate_solution(&self, grid: &crate::SudokuGrid) -> bool {
        let mut sum = 0;
        for &(row, col) in &self.cells {
            let val = grid.get_cell(row, col);
            if val == 0 {
                return false; // If any cell is empty, solution is invalid
            }
            sum += val;
        }
        sum == self.total
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

        // 1. Gather curent values in the cage
        let mut used = HashSet::new();
        let mut empty_cells = vec![];
        let mut current_sum = 0;
        for &(r, c) in &self.cells {
            let v = grid.get_cell(r, c);
            if v == 0 {
                empty_cells.push((r, c));
            } else {
                used.insert(v);
                current_sum += v;
            }
        }

        // 2. If no empty cells, return empty
        if empty_cells.is_empty() {
            return HashMap::new();
        }

        // 3. For each empty cell, collect possible values from all valid combinations
        let mut possibilities = HashMap::new();
        for &(r, c) in &empty_cells {
            possibilities.insert((r, c), HashSet::new());
        }

        // 4. Generate all combinations of unique digits (not in used), of length empty_cells.len(),
        //    that sum to (self.total - current_sum)
        let available: Vec<u8> = (1..=9).filter(|d| !used.contains(d)).collect();
        let target_sum = self.total.saturating_sub(current_sum);
        let n = empty_cells.len();

        // Recursive helper to generate combinations
        fn backtrack(
            available: &[u8],
            n: usize,
            target_sum: u8,
            start: usize,
            current: &mut Vec<u8>,
            all: &mut Vec<Vec<u8>>,
        ) {
            if n == 0 {
                if target_sum == 0 {
                    all.push(current.clone());
                }
                return;
            }
            for i in start..available.len() {
                let d = available[i];
                if d > target_sum {
                    break;
                }
                current.push(d);
                backtrack(available, n - 1, target_sum - d, i + 1, current, all);
                current.pop();
            }
        }

        let mut all_combos = vec![];
        backtrack(&available, n, target_sum, 0, &mut vec![], &mut all_combos);

        // 5. For each combo, add each digit to the corresponding cell's set
        for combo in all_combos {
            for perm in combo.iter().permutations(empty_cells.len()).unique() {
                for (i, &(r, c)) in empty_cells.iter().enumerate() {
                    possibilities.get_mut(&(r, c)).unwrap().insert(*perm[i]);
                }
            }
        }

        // 6. Convert HashSet<u8> to Vec<u8> for output
        possibilities
            .into_iter()
            .map(|(k, v)| {
                let mut vec: Vec<u8> = v.into_iter().collect();
                vec.sort_unstable();
                (k, vec)
            })
            .collect()
    }
}

impl std::fmt::Display for KillerCage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cell_str = self
            .cells
            .iter()
            .map(|&(r, c)| format!("({r}, {c}"))
            .collect::<Vec<_>>()
            .join(", ");
        write!(
            f,
            "Killer Cage [{cell_str}] Sum = {}",
            &self.total.to_string()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod possible_values {
        use super::*;

        #[test]
        fn test_possible_values() {
            let tests = [
                (vec![(0, 0), (0, 1)], 17, HashSet::from_iter(vec![8, 9])),
                (
                    vec![(0, 0), (0, 1), (0, 2)],
                    6,
                    HashSet::from_iter(vec![1, 2, 3]),
                ),
                (
                    vec![(0, 0), (0, 1)],
                    10,
                    HashSet::from_iter(vec![1, 2, 3, 4, 6, 7, 8, 9]),
                ),
            ];

            for (idx, (cells, sum, expected_possible_values)) in tests.iter().enumerate() {
                let cage = KillerCage::new(cells.clone(), *sum);
                assert_eq!(
                    cage.possible_values,
                    *expected_possible_values,
                    "Test {} failed. Expected possible values: {:?}. Got: {:?}",
                    idx + 1,
                    expected_possible_values,
                    cage.possible_values
                );
            }
        }

        #[test]
        fn test_possible_values_sum_15_three_cells() {
            let cage = KillerCage::new(vec![(0, 0), (0, 1), (0, 2)], 15);
            // All 3-digit combinations adding to 15 with distinct digits from 1-9
            let expected: HashSet<u8> = [1, 2, 3, 4, 5, 6, 7, 8, 9]
                .iter()
                .filter(|&&x| {
                    [1, 2, 3, 4, 5, 6, 7, 8, 9]
                        .iter()
                        .filter(|&&y| y != x)
                        .flat_map(|&y| {
                            [1, 2, 3, 4, 5, 6, 7, 8, 9]
                                .iter()
                                .filter(move |&&z| z != x && z != y)
                                .map(move |&z| x + y + z == 15)
                        })
                        .any(|b| b)
                })
                .copied()
                .collect();
            assert_eq!(cage.possible_values, expected);
        }
    }

    mod is_valid {
        use crate::{KillerCage, SudokuGrid, variant::Variant};

        #[test]
        fn test_value_not_in_possible_values() {
            let grid = SudokuGrid::empty();
            let cage = KillerCage::new(vec![(0, 0), (0, 1)], 10);
            assert!(!cage.is_valid(&grid, 0, 0, 5));
        }

        #[test]
        fn test_repeated_value_in_cage() {
            let mut grid = SudokuGrid::empty();
            grid.set_cell(0, 1, 2);
            let cage = KillerCage::new(vec![(0, 0), (0, 1), (0, 2)], 8);
            assert!(!cage.is_valid(&grid, 0, 0, 2));
        }

        #[test]
        fn test_sum_too_large() {
            let mut grid = SudokuGrid::empty();
            grid.set_cell(0, 1, 2);
            let cage = KillerCage::new(vec![(0, 0), (0, 1)], 3);
            assert!(!cage.is_valid(&grid, 0, 0, 5)); // 5 + 2 = 7 > 3
        }

        #[test]
        fn test_partial_sum_ok() {
            let mut grid = SudokuGrid::empty();
            grid.set_cell(0, 1, 2);
            let cage = KillerCage::new(vec![(0, 0), (0, 1)], 5);
            assert!(cage.is_valid(&grid, 0, 0, 3)); // 3 + 2 = 5
        }

        #[test]
        fn test_current_move_outside_cage() {
            let grid = SudokuGrid::empty();
            let cage = KillerCage::new(vec![(0, 0), (0, 1)], 3);
            assert!(cage.is_valid(&grid, 1, 1, 5));
        }
    }

    mod validate_solution {
        use crate::{KillerCage, SudokuGrid, variant::Variant};

        #[test]
        fn test_valid() {
            let mut grid = SudokuGrid::empty();
            grid.set_cell(0, 0, 1);
            grid.set_cell(0, 1, 2);
            let cage = KillerCage::new(vec![(0, 0), (0, 1)], 3);
            assert!(cage.validate_solution(&grid));
        }

        #[test]
        fn test_incomplete() {
            let mut grid = SudokuGrid::empty();
            grid.set_cell(0, 0, 1);
            let cage = KillerCage::new(vec![(0, 0), (0, 1)], 3);
            assert!(!cage.validate_solution(&grid));
        }

        #[test]
        fn test_incorrect_sum() {
            let mut grid = SudokuGrid::empty();
            grid.set_cell(0, 0, 1);
            grid.set_cell(0, 1, 2);
            let cage = KillerCage::new(vec![(0, 0), (0, 1)], 4);
            assert!(!cage.validate_solution(&grid));
        }
    }

    mod constrained_cells {
        use crate::{KillerCage, variant::Variant};

        #[test]
        fn test_constrained_cells() {
            let cage = KillerCage::new(vec![(1, 2), (3, 4)], 10);
            let expected = vec![(1, 2), (3, 4)];
            assert_eq!(cage.constrained_cells(), expected);
        }
    }

    mod parsing {
        use crate::{KillerCage, SudokuVariant};

        #[test]
        fn test_parse_killer_cage() {
            if let Some(SudokuVariant::Killer(k)) = KillerCage::parse("((0, 0), (0, 1)): 10") {
                assert_eq!(k.total, 10);
                assert_eq!(k.cells, vec![(0, 0), (0, 1)]);
            } else {
                panic!("Failed to parse valid KillerCage string");
            }
        }
    }

    mod get_possibilities {
        use super::KillerCage;

        use crate::{SudokuGrid, variant::Variant};

        #[test]
        fn test_cell_not_in_cage() {
            let cage = KillerCage::new(vec![(0, 0), (0, 1)], 10);
            let mut grid = SudokuGrid::empty();
            grid.set_cell(2, 2, 5); // Not in cage
            let result = cage.get_possibilities(&grid, 2, 2);
            assert!(result.is_empty());
        }

        #[test]
        fn test_get_possibilities_basic() {
            let mut grid = SudokuGrid::empty();
            grid.set_cell(0, 0, 1);
            let cage = KillerCage::new(vec![(0, 0), (0, 1)], 4);

            let result = cage.get_possibilities(&grid, 0, 0);
            let expected: Vec<u8> = vec![3];

            assert_eq!(result.len(), 1);
            assert_eq!(result.get(&(0, 1)), Some(&expected));
        }

        #[test]
        fn test_invalid_move_exceeds_sum() {
            let mut grid = SudokuGrid::empty();
            grid.set_cell(0, 0, 5); // Too big for sum of 4
            let cage = KillerCage::new(vec![(0, 0), (0, 1)], 4);

            let result = cage.get_possibilities(&grid, 0, 0);
            assert_eq!(result.len(), 1);
            assert!(result.get(&(0, 1)).unwrap().is_empty());
        }

        #[test]
        fn test_three_cells_partial_assignment() {
            let mut grid = SudokuGrid::empty();
            grid.set_cell(0, 0, 3);
            let cage = KillerCage::new(vec![(0, 0), (0, 1), (0, 2)], 10);

            let result = cage.get_possibilities(&grid, 0, 0);
            // Valid remaining pairs that sum to 7 and don’t contain 3: (1,6), (2,5), (4,3), (5,2), etc.
            // But 3 already used, so (4,3) and (3,4) are invalid
            let expected = vec![1, 2, 5, 6];
            assert_eq!(result.len(), 2);
            assert_eq!(result.get(&(0, 1)), Some(&expected));
            assert_eq!(result.get(&(0, 2)), Some(&expected));
        }

        #[test]
        fn test_cage_filled() {
            let mut grid = SudokuGrid::empty();
            grid.set_cell(0, 0, 3);
            grid.set_cell(0, 1, 7);
            let cage = KillerCage::new(vec![(0, 0), (0, 1), (0, 2)], 10);
            let result = cage.get_possibilities(&grid, 0, 1);
            assert_eq!(result.len(), 1);
            assert!(result.get(&(0, 2)).unwrap().is_empty());
        }
    }
}
