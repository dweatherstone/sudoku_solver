use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::{SudokuVariant, file_parser::parse_positions, variant::Variant};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Arrow {
    cells: Vec<(usize, usize)>,
}

impl Arrow {
    pub fn new(cells: Vec<(usize, usize)>) -> Self {
        Arrow { cells }
    }

    pub fn parse(data: &str) -> Option<SudokuVariant> {
        let cells = parse_positions(data).ok()?;
        Some(SudokuVariant::Arrow(Arrow::new(cells)))
    }
}

impl Variant for Arrow {
    fn is_valid(&self, grid: &crate::SudokuGrid, row: usize, col: usize, value: u8) -> bool {
        if !self.cells.contains(&(row, col)) {
            return true;
        }
        // Clone current values, and insert proposed value
        let mut values = self
            .cells
            .iter()
            .map(|&(r, c)| grid.get_cell(r, c))
            .collect::<Vec<u8>>();

        // Find the index of (row, col) in the arrow
        if let Some(pos) = self.cells.iter().position(|&(r, c)| r == row && c == col) {
            values[pos] = value;
        }

        let head_value = values[0];
        let body_values = &values[1..];

        let known_sum: u8 = body_values.iter().sum();
        let unknown_count = body_values.iter().filter(|&&v| v == 0).count();

        // If the head cell is 0 (unknown), we can only check whether the body can *possibly* sum to a valid head (<=9)
        if head_value == 0 {
            // If body is fully filled but head is unknown, we can't validate yet
            if unknown_count == 0 {
                // Head must be equal to the known body sum and nonzero
                return known_sum <= 9;
            }
            // Otherwise, just check that the body sum is still in the realm of possibility
            // (realistically not needed unless you want to prune impossible sums)
            return true;
        }

        // Head is known, apply tighter constraint
        if known_sum > head_value {
            return false; // overshoot
        }

        // Miniumum possible sum for body must be <= head
        let minimum_possible_sum = known_sum + unknown_count as u8;
        if minimum_possible_sum > head_value {
            return false;
        }

        // Final check: if all body digits are known, they must sum to head
        if unknown_count == 0 {
            return known_sum == head_value;
        }

        // Stil consistent
        true
    }

    fn validate_solution(&self, grid: &crate::SudokuGrid) -> bool {
        if self.cells.len() < 2 {
            return false;
        }
        let head = self.cells[0];
        let head_value = grid.get_cell(head.0, head.1);
        if head_value == 0 {
            return false;
        }
        let body_values = self
            .cells
            .iter()
            .skip(1)
            .map(|&(r, c)| grid.get_cell(r, c))
            .collect::<Vec<_>>();
        if body_values.contains(&0) {
            return false;
        }

        body_values.iter().sum::<u8>() == head_value
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
        // For each unknown cell on the arrow, return all values (1..=9) that can participate in at least one valid assignment (with the other unknowns) that satisfies the arrow sum, given the current grid state. No uniqueness filtering is applied.

        // If (row, col) not on the arrow, just return
        let _ = match self.cells.iter().position(|&(r, c)| r == row && c == col) {
            Some(i) => i,
            None => return HashMap::new(),
        };

        let mut possibilities: HashMap<(usize, usize), Vec<u8>> = HashMap::new();

        // Gather current values for all cells on the arrow
        let values: Vec<u8> = self
            .cells
            .iter()
            .map(|&(r, c)| grid.get_cell(r, c))
            .collect();

        // Identify unknown cells (value == 0)
        let unknowns: Vec<_> = self
            .cells
            .iter()
            .zip(values.iter())
            .filter(|&(_, &v)| v == 0)
            .map(|(&(r, c), _)| (r, c))
            .collect();

        if unknowns.is_empty() {
            // All cells are known, nothing to do
            return HashMap::new();
        }

        // For each unknown, domain is simply 1..=9 (no uniqueness filtering)
        let domains: Vec<Vec<u8>> = vec![(1..=9).collect(); unknowns.len()];

        // For each possible assignment to the unknowns, check if it satisfies the arrow constraint
        let mut cell_poss: HashMap<(usize, usize), HashSet<u8>> = HashMap::new();
        for assignment in domains.iter().multi_cartesian_product() {
            // Fill in the unknowns with this assignment
            let mut test_values = values.clone();
            for (&cell, &&val) in unknowns.iter().zip(assignment.iter()) {
                let pos = self.cells.iter().position(|&c| c == cell).unwrap();
                test_values[pos] = val;
            }
            let head_value = test_values[0];
            let body_sum: u8 = test_values.iter().skip(1).sum();

            // Check the arrow constraint
            if head_value != 0 && body_sum != head_value {
                continue;
            }
            if head_value == 0 && body_sum > 9 {
                continue;
            }

            // If valid, record these values as possible for each cell
            for (&cell, &&val) in unknowns.iter().zip(assignment.iter()) {
                cell_poss.entry(cell).or_default().insert(val);
            }
        }

        // Convert HashSet<u8> to Vec<u8> for output
        for (cell, vals) in cell_poss {
            let mut v: Vec<u8> = vals.into_iter().collect();
            v.sort_unstable();
            possibilities.insert(cell, v);
        }

        // For known cells, their only possible value is their current value
        for &(r, c) in self.cells.iter() {
            let v = grid.get_cell(r, c);
            if v != 0 && (r, c) != (row, col) {
                possibilities.insert((r, c), vec![v]);
            }
        }

        possibilities
    }
}

impl std::fmt::Display for Arrow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::from("Arrow: head = ");
        output.push_str(&format!("({}, {})", self.cells[0].0, self.cells[0].1));
        output.push_str(", arrow: [");
        output.push_str(
            self.cells
                .iter()
                .skip(1)
                .map(|&(r, c)| format!("({r}, {c})"))
                .collect::<Vec<_>>()
                .join(", ")
                .as_str(),
        );
        output.push(']');
        write!(f, "{output}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SudokuGrid;

    fn setup_arrow() -> Arrow {
        // Arrow from (0,0) (head) to (0,1) and (0,2)
        Arrow::new(vec![(0, 0), (0, 1), (0, 2)])
    }

    #[test]
    fn test_constrained_cells() {
        let arrow = setup_arrow();
        assert_eq!(arrow.constrained_cells(), vec![(0, 0), (0, 1), (0, 2)]);
    }

    #[test]
    fn test_is_valid_head_correct_sum() {
        let arrow = setup_arrow();
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 1, 3);
        grid.set_cell(0, 2, 4);
        // Head should be 7
        assert!(arrow.is_valid(&grid, 0, 0, 7));
    }

    #[test]
    fn test_is_valid_head_incorrect_sum() {
        let arrow = setup_arrow();
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 1, 3);
        grid.set_cell(0, 2, 4);
        // Head is 8, but sum is 7
        assert!(!arrow.is_valid(&grid, 0, 0, 8));
    }

    #[test]
    fn test_is_valid_body_cell() {
        let arrow = setup_arrow();
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 0, 7);
        grid.set_cell(0, 2, 4);
        // (0,1) is 3, so 3+4=7, valid
        assert!(arrow.is_valid(&grid, 0, 1, 3));
        // (0,1) is 2, so 2+4=6 != 7, invalid
        assert!(!arrow.is_valid(&grid, 0, 1, 2));
    }

    #[test]
    fn test_validate_solution_valid() {
        let arrow = setup_arrow();
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 0, 7);
        grid.set_cell(0, 1, 3);
        grid.set_cell(0, 2, 4);
        assert!(arrow.validate_solution(&grid));
    }

    #[test]
    fn test_validate_solution_invalid() {
        let arrow = setup_arrow();
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 0, 8);
        grid.set_cell(0, 1, 3);
        grid.set_cell(0, 2, 4);
        assert!(!arrow.validate_solution(&grid));
    }

    #[test]
    fn test_get_possibilities_not_on_arrow() {
        let arrow = setup_arrow();
        let grid = SudokuGrid::empty();
        let result = arrow.get_possibilities(&grid, 1, 1);
        assert!(result.is_empty());
    }

    #[test]
    fn test_get_possibilities_on_arrow_head() {
        let arrow = setup_arrow();
        let mut grid = SudokuGrid::empty();
        // Suppose we just set (0,0) to 7
        grid.set_cell(0, 0, 7);
        let result = arrow.get_possibilities(&grid, 0, 0);
        // For each body cell, possible values are those (1..=9) such that sum of two is 7 and both are 1..=9
        // For (0,1) and (0,2), possible pairs: (1,6),(2,5),(3,4),(4,3),(5,2),(6,1)
        // So for (0,1): [1,2,3,4,5,6], for (0,2): [1,2,3,4,5,6]
        assert!(result.contains_key(&(0, 1)));
        assert!(result.contains_key(&(0, 2)));
        for v in result.get(&(0, 1)).unwrap() {
            assert!(1 <= *v && *v <= 6);
        }
        for v in result.get(&(0, 2)).unwrap() {
            assert!(1 <= *v && *v <= 6);
        }
    }

    #[test]
    fn test_get_possibilities_on_arrow_body() {
        let arrow = setup_arrow();
        let mut grid = SudokuGrid::empty();
        // Suppose we just set (0,1) to 3, and head is 7
        grid.set_cell(0, 0, 7);
        grid.set_cell(0, 1, 3);
        let result = arrow.get_possibilities(&grid, 0, 1);
        // (0,2) must be 4
        assert_eq!(result.get(&(0, 2)), Some(&vec![4]));
        assert_eq!(result.get(&(0, 0)), Some(&vec![7]));
    }

    #[test]
    fn test_arrow_with_two_cells() {
        let mut grid = SudokuGrid::empty();
        let arrow = Arrow::new(vec![(2, 2), (3, 3)]);
        // First check that all values are possible for both cells
        let result = arrow.get_possibilities(&grid, 2, 2);
        assert_eq!(result.len(), 2);
        assert!(result.contains_key(&(2, 2)));
        assert!(result.contains_key(&(3, 3)));
        for v in result.get(&(2, 2)).unwrap() {
            assert!(1 <= *v && *v <= 9);
        }
        for v in result.get(&(3, 3)).unwrap() {
            assert!(1 <= *v && *v <= 9);
        }
        // Now set (2,2) to 5 and check that (3, 3) must also be 5
        grid.set_cell(2, 2, 5);
        let result = arrow.get_possibilities(&grid, 2, 2);
        assert_eq!(result.len(), 1);
        assert!(result.contains_key(&(3, 3)));
        assert_eq!(result.get(&(3, 3)).unwrap(), &vec![5]);
        // Now check that setting the other cell on the arrow also works as expected.
        grid.set_cell(2, 2, 0);
        grid.set_cell(3, 3, 4);
        let result = arrow.get_possibilities(&grid, 2, 2);
        assert_eq!(result.len(), 2);
        assert!(result.contains_key(&(2, 2)));
        assert!(result.contains_key(&(3, 3)));
        assert_eq!(result.get(&(2, 2)).unwrap(), &vec![4]);
        assert_eq!(result.get(&(3, 3)).unwrap(), &vec![4]);
    }

    #[test]
    fn test_arrow_all_cells_unknown() {
        // Arrow with 3 cells, all unknown
        let arrow = setup_arrow();
        let grid = SudokuGrid::empty();
        let result = arrow.get_possibilities(&grid, 0, 0);
        // All cells should have all values 1..=9 as possible
        for cell in &[(0, 1), (0, 2)] {
            assert_eq!(result.get(cell).unwrap(), &(1..=8).collect::<Vec<u8>>());
        }
        assert_eq!(result.get(&(0, 0)).unwrap(), &vec![2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_arrow_head_set_multiple_body_unknown() {
        // Head is set, all body cells unknown
        let arrow = setup_arrow();
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 0, 5);
        let result = arrow.get_possibilities(&grid, 0, 0);
        // Only pairs of body values that sum to 5 are possible
        let mut possible_pairs = vec![];
        for a in 1..=9 {
            for b in 1..=9 {
                if a + b == 5 {
                    possible_pairs.push((a, b));
                }
            }
        }
        let mut expected_01 = possible_pairs
            .iter()
            .map(|(a, _)| *a)
            .collect::<HashSet<_>>();
        let mut expected_02 = possible_pairs
            .iter()
            .map(|(_, b)| *b)
            .collect::<HashSet<_>>();
        let res_01 = result
            .get(&(0, 1))
            .unwrap()
            .iter()
            .cloned()
            .collect::<HashSet<_>>();
        let res_02 = result
            .get(&(0, 2))
            .unwrap()
            .iter()
            .cloned()
            .collect::<HashSet<_>>();
        assert_eq!(res_01, expected_01);
        assert_eq!(res_02, expected_02);
    }

    #[test]
    fn test_arrow_some_body_set_head_unknown() {
        // Some body cells set, head unknown
        let arrow = setup_arrow();
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 1, 2);
        // Head and (0,2) unknown
        let result = arrow.get_possibilities(&grid, 0, 2);
        // Head must be 2 + (0,2), so for each possible (0,2), head is 2 + v
        for v in 1..=9 {
            let head_val = 2 + v;
            if head_val <= 9 {
                assert!(result.get(&(0, 0)).unwrap().contains(&head_val));
                assert!(result.get(&(0, 2)).unwrap().contains(&v));
            }
        }
    }

    #[test]
    fn test_arrow_impossible_sum() {
        // Set body cells so their sum is greater than 9
        let arrow = setup_arrow();
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 1, 8);
        grid.set_cell(0, 2, 5);
        // Head is unknown, but sum is 13 > 9, so no valid head
        let result = arrow.get_possibilities(&grid, 0, 0);
        assert!(result.get(&(0, 0)).unwrap_or(&vec![]).is_empty());
    }

    #[test]
    fn test_arrow_one_cell_left() {
        // All but one cell are filled
        let arrow = setup_arrow();
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 0, 9);
        grid.set_cell(0, 1, 4);
        // Only (0,2) is unknown, must be 5
        let result = arrow.get_possibilities(&grid, 0, 2);
        assert_eq!(result.get(&(0, 2)), Some(&vec![5]));
    }

    #[test]
    fn test_arrow_repeated_digits_allowed() {
        // Set up a case where repeated digits are required
        let arrow = Arrow::new(vec![(0, 0), (0, 1), (0, 2)]);
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 0, 4);
        // Only (2,2) and (2,2) = (2,2) is not on the arrow, so test repeated digits
        // For (0,1) and (0,2), possible pairs: (2,2)
        let result = arrow.get_possibilities(&grid, 0, 0);
        assert!(result.get(&(0, 1)).unwrap().contains(&2));
        assert!(result.get(&(0, 2)).unwrap().contains(&2));
    }

    #[test]
    fn test_arrow_head_set_to_zero_invalid() {
        // Head is set to 0 (invalid)
        let arrow = setup_arrow();
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 0, 0);
        let result = arrow.get_possibilities(&grid, 0, 0);
        // All body cells should have all values 1..=9 as possible (since head is unknown/invalid)
        for cell in &[(0, 1), (0, 2)] {
            assert_eq!(result.get(cell).unwrap(), &(1..=8).collect::<Vec<u8>>());
        }
    }
}
