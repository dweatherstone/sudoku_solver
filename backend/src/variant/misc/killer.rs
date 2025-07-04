use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{SudokuVariant, file_parser::parse_positions, variant::Variant};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct KillerCage {
    cells: Vec<(usize, usize)>,
    sum: u32,
    possible_values: HashSet<u8>,
}

impl KillerCage {
    pub fn new(cells: Vec<(usize, usize)>, sum: u32) -> Self {
        let mut cage = KillerCage {
            cells,
            sum,
            possible_values: HashSet::new(),
        };
        cage.set_possible_values();
        cage
    }

    // pub fn cells(&self) -> &Vec<(usize, usize)> {
    //     &self.cells
    // }

    // pub fn sum(&self) -> u32 {
    //     self.sum
    // }

    pub fn parse(data: &str) -> Option<SudokuVariant> {
        let parts: Vec<&str> = data.split(':').collect();
        if parts.len() != 2 {
            return None;
        }
        let cells = parse_positions(parts[0].trim()).ok()?;
        let sum = parts[1].trim().parse().ok()?;
        Some(SudokuVariant::Killer(KillerCage::new(cells, sum)))
    }

    fn set_possible_values(&mut self) {
        let digits = (1u8..=9).collect::<Vec<_>>();
        let mut result = HashSet::new();

        // Recursive helper to generate combinations
        fn backtrack(
            digits: &[u8],
            size: usize,
            target_sum: u32,
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
                let d = digits[i] as u32;
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
            self.sum,
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
            current_sum += if (r, c) == (row, col) {
                value as u32
            } else {
                val as u32
            };
        }

        if empty_cells == 0 {
            current_sum == self.sum
        } else {
            current_sum <= self.sum
        }
    }

    fn validate_solution(&self, grid: &crate::SudokuGrid) -> bool {
        let mut sum = 0;
        for &(row, col) in &self.cells {
            let val = grid.get_cell(row, col);
            if val == 0 {
                return false; // If any cell is empty, solution is invalid
            }
            sum += val as u32;
        }
        sum == self.sum
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
        unimplemented!()
    }
}

impl std::fmt::Display for KillerCage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::from("Killer Cage [");
        output.push_str(
            self.cells
                .iter()
                .map(|&(r, c)| format!("({}, {})", r, c))
                .collect::<Vec<_>>()
                .join(", ")
                .as_str(),
        );
        output.push_str("] Sum = ");
        output.push_str(&self.sum.to_string());
        write!(f, "{}", output)
    }
}

#[cfg(test)]
mod tests {
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
}
