use std::{
    cmp::{max, min},
    collections::{HashMap, HashSet},
};

use serde::{Deserialize, Serialize};

use crate::{SudokuVariant, file_parser::parse_positions, variant::Variant};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Renban {
    cells: Vec<(usize, usize)>,
}

impl Renban {
    pub fn new(cells: Vec<(usize, usize)>) -> Self {
        Renban { cells }
    }

    pub fn parse(data: &str) -> Option<SudokuVariant> {
        let positions = parse_positions(data).ok()?;
        Some(SudokuVariant::Renban(Renban::new(positions)))
    }
}

impl Variant for Renban {
    fn is_valid(&self, grid: &crate::SudokuGrid, row: usize, col: usize, value: u8) -> bool {
        // If the proposed cell is not in the Renban's cells, then return true
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

        // If current cells is empty, then there is nothing to constrain, so return early
        if filled_cells.is_empty() {
            return true;
        }

        // Add the proposed value to the current_cells
        filled_cells.push(value);

        // If the line would be complete, then use the validate_solution logic
        if filled_cells.len() == self.cells.len() {
            let mut proposed_grid = grid.clone();
            proposed_grid.set_cell(row, col, value);
            return self.validate_solution(&proposed_grid);
        }

        let n = self.cells.len() as i8;
        // Can use unwrap here, as we know that current_cells at least has the proposed value
        let min_current = *filled_cells.iter().min().unwrap() as i8;
        let max_current = *filled_cells.iter().max().unwrap() as i8;
        let span = max_current - min_current + 1;
        if span > n {
            return false;
        }
        if max(1, max_current - n + 1) > min(9 - n + 1, min_current) {
            return false;
        }

        true
    }

    fn constrained_cells(&self) -> Vec<(usize, usize)> {
        self.cells.clone()
    }

    fn validate_solution(&self, grid: &crate::SudokuGrid) -> bool {
        let mut values = HashSet::new();
        let mut min_val = 9;
        let mut max_val = 1;

        for &(row, col) in &self.cells {
            let value = grid.get_cell(row, col);
            if value == 0 || !values.insert(value) {
                // duplicate value or zero
                return false;
            }
            min_val = min(min_val, value);
            max_val = max(max_val, value);
        }

        // Get min and max values and then check that the values are continuous.
        if max_val - min_val + 1 != self.cells.len() as u8 {
            return false;
        }
        // Check that the set of values is the same as the expected set based on the min and max values
        HashSet::from_iter(min_val..=max_val) == values
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

        let mut known: HashMap<(usize, usize), u8> = HashMap::new();
        for &(r, c) in &self.cells {
            let val = grid.get_cell(r, c);
            if val != 0 {
                // Duplicate check
                if known.values().any(|&v| v == val) {
                    return HashMap::new();
                }
                known.insert((r, c), val);
            }
        }

        let known_values: HashSet<u8> = known.values().copied().collect();
        let line_len = self.cells.len() as u8;

        // Check for invalid spread
        if known_values.len() > 1 {
            let min = *known_values.iter().min().unwrap();
            let max = *known_values.iter().max().unwrap();
            if max - min + 1 > line_len {
                return HashMap::new();
            }
        }

        // Generate all valid renban ranges of required length
        let mut valid_sets: Vec<HashSet<u8>> = Vec::new();
        for start in 1..=(10 - line_len) {
            let candidate: HashSet<u8> = (start..start + line_len).collect();
            if known_values.is_subset(&candidate) {
                valid_sets.push(candidate);
            }
        }

        // Union of all possible values from those sets (excluding known)
        let mut allowed_values = HashSet::new();
        for s in &valid_sets {
            for v in s {
                if !known_values.contains(v) {
                    allowed_values.insert(*v);
                }
            }
        }

        let mut possibilities = HashMap::new();
        for &(r, c) in &self.cells {
            if grid.get_cell(r, c) != 0 {
                continue;
            }
            possibilities.insert((r, c), {
                let mut v: Vec<u8> = allowed_values.iter().copied().collect();
                v.sort_unstable();
                v
            });
        }
        possibilities
    }
}

impl std::fmt::Display for Renban {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cells_str = self
            .cells
            .iter()
            .map(|&(r, c)| format!("({r}, {c})"))
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "Renban Line [{cells_str}]")
    }
}

#[cfg(test)]
mod tests {
    use crate::{SudokuGrid, variant::Variant};

    use super::Renban;

    #[test]
    fn test_get_possibilities_basic() {
        let renban = Renban::new(vec![(0, 0), (0, 1), (0, 2), (0, 3), (0, 4)]);
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 0, 3);
        let result = renban.get_possibilities(&grid, 0, 0);
        let expected: Vec<u8> = vec![1, 2, 4, 5, 6, 7];
        assert_eq!(result.len(), 4);
        for c in 1..5 {
            assert_eq!(result.get(&(0, c)).unwrap(), &expected);
        }
    }

    #[test]
    fn test_get_possibilities_two_givens() {
        let renban = Renban::new(vec![(0, 0), (0, 1), (0, 2), (0, 3)]);
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 0, 5);
        grid.set_cell(0, 3, 6);
        let result = renban.get_possibilities(&grid, 0, 3);
        assert_eq!(result.len(), 2);
        assert_eq!(result.get(&(0, 1)).unwrap(), &vec![3, 4, 7, 8]);
        assert_eq!(result.get(&(0, 2)).unwrap(), &vec![3, 4, 7, 8]);
    }

    #[test]
    fn test_get_possibilities_fully_known_line() {
        let renban = Renban::new(vec![(0, 0), (0, 1), (0, 2)]);
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 0, 3);
        grid.set_cell(0, 1, 2);
        grid.set_cell(0, 2, 4);
        let result = renban.get_possibilities(&grid, 0, 2);
        assert!(result.is_empty());
    }

    #[test]
    fn test_get_possibilities_impossible_range() {
        let renban = Renban::new(vec![(0, 0), (0, 1), (0, 2)]);
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 0, 1);
        grid.set_cell(0, 1, 5);
        let result = renban.get_possibilities(&grid, 0, 1);
        assert!(result.is_empty());
    }

    #[test]
    fn test_get_possibilities_duplicates_on_line() {
        let renban = Renban::new(vec![(0, 0), (0, 1), (0, 2)]);
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 0, 5);
        grid.set_cell(0, 1, 5);
        let result = renban.get_possibilities(&grid, 0, 1);
        assert!(result.is_empty());
    }

    #[test]
    fn test_get_possibilities_edge_of_range() {
        let renban = Renban::new(vec![(0, 0), (0, 1), (0, 2)]);
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 1, 9);
        let result = renban.get_possibilities(&grid, 0, 1);
        assert_eq!(result.get(&(0, 0)).unwrap(), &vec![7, 8]);
        assert_eq!(result.get(&(0, 2)).unwrap(), &vec![7, 8]);
    }

    #[test]
    fn test_get_possibilities_highly_constrained() {
        let renban = Renban::new(vec![(0, 0), (0, 1), (0, 2)]);
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 0, 2);
        grid.set_cell(0, 1, 4);
        let result = renban.get_possibilities(&grid, 0, 1);
        assert_eq!(result.len(), 1);
        assert_eq!(result.get(&(0, 2)).unwrap(), &vec![3]);
    }

    #[test]
    fn test_valid_solution() {
        let renban = Renban::new(vec![(0, 0), (0, 1), (0, 2)]);
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 0, 6);
        grid.set_cell(0, 1, 4);
        grid.set_cell(0, 2, 5);
        assert!(
            renban.validate_solution(&grid),
            "Should be a valid solution"
        );
    }

    #[test]
    fn test_solution_non_consecutive() {
        let renban = Renban::new(vec![(0, 0), (0, 1), (0, 2)]);
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 0, 6);
        grid.set_cell(0, 1, 4);
        grid.set_cell(0, 2, 7);
        assert!(!renban.validate_solution(&grid), "Should be invlid");
    }

    #[test]
    fn test_solution_duplicate() {
        let renban = Renban::new(vec![(1, 0), (1, 1), (1, 2)]);
        let mut grid = SudokuGrid::empty();
        grid.set_cell(1, 0, 2);
        grid.set_cell(1, 1, 2);
        grid.set_cell(1, 2, 3);
        assert!(!renban.validate_solution(&grid), "Should be invlid");
    }

    #[test]
    fn test_valid_proposal() {
        let renban = Renban::new(vec![(0, 0), (0, 1), (0, 2)]);
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 1, 4);
        grid.set_cell(0, 2, 5);
        assert!(renban.is_valid(&grid, 0, 0, 6), "Should be valid proposal");
        assert!(renban.is_valid(&grid, 0, 0, 3), "Should be valid proposal");
    }

    #[test]
    fn test_invalid_proposal_duplicate() {
        let renban = Renban::new(vec![(0, 0), (0, 1), (0, 2)]);
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 1, 4);
        grid.set_cell(0, 2, 5);
        assert!(!renban.is_valid(&grid, 0, 0, 4), "Would cause a duplicate");
    }

    #[test]
    fn test_invalid_proposal_impossible_sequence() {
        let renban = Renban::new(vec![(0, 0), (0, 1), (0, 2)]);
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 1, 4);
        grid.set_cell(0, 2, 5);
        assert!(
            !renban.is_valid(&grid, 0, 0, 7),
            "Valid sequence impossible"
        );
        assert!(
            !renban.is_valid(&grid, 0, 0, 2),
            "Valid sequence impossible"
        );
    }

    #[test]
    fn test_valid_proposal_incomplete() {
        let renban = Renban::new(vec![(0, 0), (0, 1), (0, 2), (0, 3)]);
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 1, 4);
        grid.set_cell(0, 2, 5);
        assert!(renban.is_valid(&grid, 0, 0, 6), "Should be valid proposal");
        assert!(renban.is_valid(&grid, 0, 0, 3), "Should be valid proposal");
        assert!(renban.is_valid(&grid, 0, 3, 7), "Should be valid proposal");
        assert!(renban.is_valid(&grid, 0, 3, 2), "Should be valid proposal");
    }

    #[test]
    fn test_invalid_proposal_incomplete() {
        let renban = Renban::new(vec![(0, 0), (0, 1), (0, 2), (0, 3)]);
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 1, 4);
        grid.set_cell(0, 2, 5);
        assert!(
            !renban.is_valid(&grid, 0, 0, 8),
            "Should be invalid proposal"
        );
        assert!(
            !renban.is_valid(&grid, 0, 0, 1),
            "Should be invalid proposal"
        );
        assert!(
            !renban.is_valid(&grid, 0, 3, 4),
            "Should be invalid proposal"
        );
        assert!(
            !renban.is_valid(&grid, 0, 3, 5),
            "Should be invalid proposal"
        );
    }

    #[test]
    fn test_single_cell_valid() {
        let renban = Renban::new(vec![(4, 4)]);
        let mut grid = SudokuGrid::empty();
        grid.set_cell(4, 4, 7);
        assert!(renban.validate_solution(&grid));
    }

    #[test]
    fn test_length_9_renban() {
        let renban = Renban::new(vec![
            (0, 0),
            (1, 1),
            (2, 2),
            (3, 3),
            (4, 4),
            (5, 5),
            (6, 6),
            (7, 7),
            (8, 8),
        ]);
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 0, 3);
        grid.set_cell(1, 1, 4);
        grid.set_cell(2, 2, 5);
        grid.set_cell(3, 3, 6);
        grid.set_cell(4, 4, 7);
        grid.set_cell(5, 5, 8);
        grid.set_cell(6, 6, 9);
        // Check that both 1 and 2 could be added to the renban
        assert!(renban.is_valid(&grid, 7, 7, 1));
        assert!(renban.is_valid(&grid, 7, 7, 2));
        // Check that 3 and 9 are invalid
        assert!(!renban.is_valid(&grid, 7, 7, 3));
        assert!(!renban.is_valid(&grid, 7, 7, 9));
        // Check that this is not a valid solution as there are empty cells
        assert!(!renban.validate_solution(&grid));
        // Set a value to 1, and check that the remaining cell can only be 2
        grid.set_cell(8, 8, 1);
        assert!(renban.is_valid(&grid, 7, 7, 2));
        assert!(!renban.is_valid(&grid, 7, 7, 1));
        assert!(!renban.validate_solution(&grid));
        // Set the final cell and check that solution is valid
        grid.set_cell(7, 7, 2);
        assert!(renban.validate_solution(&grid));
    }

    #[test]
    fn test_proposal_underflow() {
        let renban = Renban::new(vec![
            (0, 0),
            (0, 1),
            (0, 2),
            (0, 3),
            (0, 4),
            (0, 5),
            (0, 6),
            (0, 7),
        ]);
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 1, 1);

        assert!(renban.is_valid(&grid, 0, 2, 2));
    }
}
