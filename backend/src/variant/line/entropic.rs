use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{SudokuVariant, file_parser::parse_positions, variant::Variant};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Entropic {
    cells: Vec<(usize, usize)>,
}

impl Entropic {
    pub fn new(cells: Vec<(usize, usize)>) -> Self {
        Entropic { cells }
    }

    pub fn parse(data: &str) -> Option<SudokuVariant> {
        let cells = parse_positions(data).ok()?;
        Some(SudokuVariant::Entropic(Entropic::new(cells)))
    }
}

impl Variant for Entropic {
    fn is_valid(&self, grid: &crate::SudokuGrid, row: usize, col: usize, value: u8) -> bool {
        // If the proposed cell is not on the line, then nothing to do
        if !self.cells.contains(&(row, col)) {
            return true;
        }
        // Get the current values from the grid
        let mut values = self
            .cells
            .iter()
            .map(|&(r, c)| grid.get_cell(r, c))
            .collect::<Vec<u8>>();

        // Find the index of (row, col) in the entropic line
        if let Some(pos) = self.cells.iter().position(|&(r, c)| r == row && c == col) {
            // simulate placing the value
            values[pos] = value;
        }

        // Now run the windows entropic checks
        for window in values.windows(3) {
            let bands = window.iter().map(|&v| to_entropy(v)).collect::<Vec<_>>();

            let filled = bands.iter().filter_map(|&b| b).collect::<Vec<_>>();
            let unique = filled.iter().cloned().collect::<HashSet<Entropy>>();

            match filled.len() {
                3 if unique.len() != 3 => return false,
                2 if unique.len() == 1 => return false,
                _ => {}
            }
        }

        true
    }

    fn validate_solution(&self, grid: &crate::SudokuGrid) -> bool {
        let values: Vec<u8> = self
            .cells
            .iter()
            .map(|&(r, c)| grid.get_cell(r, c))
            .collect();

        // Check all cells are filled
        if values.contains(&0) {
            return false;
        }

        for window in values.windows(3) {
            let mut has_low = false;
            let mut has_mid = false;
            let mut has_high = false;

            for &val in window {
                match to_entropy(val) {
                    Some(Entropy::Low) => has_low = true,
                    Some(Entropy::Medium) => has_mid = true,
                    Some(Entropy::High) => has_high = true,
                    None => return false, // invalid digit
                }
            }

            if !(has_low && has_mid && has_high) {
                return false;
            }
        }

        true
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
        // If (row, col) is not on the line, just pass
        if !self.cells.contains(&(row, col)) {
            return HashMap::new();
        }

        if to_entropy(grid.get_cell(row, col)).is_none() {
            return HashMap::new();
        }

        // Step 1: For each mod-3 group, determine if any cell is set, and if so, which entropy
        let mut group_entropy: [Option<Entropy>; 3] = [None, None, None];
        for (i, &(r, c)) in self.cells.iter().enumerate() {
            let val = grid.get_cell(r, c);
            if val == 0 {
                continue;
            }
            if let Some(entropy) = to_entropy(val) {
                let group = i % 3;
                if let Some(existing) = group_entropy[group] {
                    if existing != entropy {
                        // Contradiction: two different entropies in the same group
                        // TODO: Make this an error at some point!
                        println!("Contradiction in group {group}: {existing:?} vs {entropy:?}");
                        return HashMap::new();
                    }
                } else {
                    group_entropy[group] = Some(entropy);
                }
            }
        }

        // Step 2: Assign remaining entropies to unassigned groups
        let used: Vec<Entropy> = group_entropy.iter().filter_map(|&e| e).collect();
        let unused: Vec<Entropy> = [Entropy::Low, Entropy::Medium, Entropy::High]
            .iter()
            .copied()
            .filter(|e| !used.contains(e))
            .collect();

        // Step 3: For each empty cell, only allow digits from its group's assigned entropy,
        // or, if not assigned, from all unused entropies
        let mut possibilities = HashMap::new();
        for (i, &(r, c)) in self.cells.iter().enumerate() {
            if grid.get_cell(r, c) != 0 {
                continue;
            }
            let group = i % 3;
            if let Some(entropy) = group_entropy[group] {
                possibilities.insert((r, c), entropy.digit_range());
            } else {
                // Union of all unused entropies
                let mut digits = Vec::new();
                for e in &unused {
                    digits.extend(e.digit_range());
                }
                digits.sort();
                digits.dedup();
                possibilities.insert((r, c), digits);
            }
        }

        possibilities
    }
}

impl std::fmt::Display for Entropic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::from("Entropic Line [");
        output.push_str(
            self.cells
                .iter()
                .map(|&(r, c)| format!("({r}, {c})"))
                .collect::<Vec<_>>()
                .join(", ")
                .as_str(),
        );
        write!(f, "{output}")
    }
}

// fn intersect_vecs(a: &[u8], b: &[u8]) -> Vec<u8> {
//     let set_b: HashSet<_> = b.iter().copied().collect();
//     a.iter().copied().filter(|x| set_b.contains(x)).collect()
// }

#[derive(Debug, PartialEq, Eq, Clone, Hash, Copy)]
enum Entropy {
    Low,
    Medium,
    High,
}

impl Entropy {
    fn digit_range(&self) -> Vec<u8> {
        match self {
            Entropy::Low => vec![1, 2, 3],
            Entropy::Medium => vec![4, 5, 6],
            Entropy::High => vec![7, 8, 9],
        }
    }
}

fn to_entropy(value: u8) -> Option<Entropy> {
    match value {
        1..=3 => Some(Entropy::Low),
        4..=6 => Some(Entropy::Medium),
        7..=9 => Some(Entropy::High),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::Entropic;

    use crate::{SudokuGrid, variant::Variant};

    #[test]
    fn test_solution_valid() {
        let entropic = Entropic::new(vec![(0, 0), (0, 1), (0, 2), (0, 3)]);
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 0, 1);
        grid.set_cell(0, 1, 4);
        grid.set_cell(0, 2, 7);
        grid.set_cell(0, 3, 2);
        assert!(entropic.validate_solution(&grid), "Should be valid triplet");
    }

    #[test]
    fn test_solution_incomplete() {
        let entropic = Entropic::new(vec![(0, 0), (0, 1), (0, 2), (0, 3)]);
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 0, 1);
        grid.set_cell(0, 1, 4);
        grid.set_cell(0, 2, 7);
        assert!(
            !entropic.validate_solution(&grid),
            "All values need to be filled - invalid"
        );
    }

    #[test]
    fn test_solution_wrong_order() {
        let entropic = Entropic::new(vec![(0, 0), (0, 1), (0, 2), (0, 3)]);
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 0, 1);
        grid.set_cell(0, 1, 4);
        grid.set_cell(0, 2, 2);
        grid.set_cell(0, 3, 7);
        assert!(
            !entropic.validate_solution(&grid),
            "Two Low values in triplet - invalid"
        );
    }

    #[test]
    fn test_solution_valid_short() {
        let entropic = Entropic::new(vec![(0, 0), (0, 1)]);
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 0, 1);
        grid.set_cell(0, 1, 2);
        assert!(
            entropic.validate_solution(&grid),
            "Lines shorter than 3 cells should always pass"
        );
    }

    #[test]
    fn test_solution_all_same_entropy() {
        let entropic = Entropic::new(vec![(1, 0), (1, 1), (1, 2)]);
        let mut grid = SudokuGrid::empty();
        grid.set_cell(1, 0, 4);
        grid.set_cell(1, 1, 5);
        grid.set_cell(1, 2, 6);
        assert!(
            !entropic.validate_solution(&grid),
            "All medium values - invalid"
        );
    }

    #[test]
    fn test_valid_proposal_in_window() {
        let entropic = Entropic::new(vec![(0, 0), (0, 1), (0, 2)]);
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 0, 2);
        grid.set_cell(0, 1, 5);
        assert!(
            entropic.is_valid(&grid, 0, 2, 9),
            "Should complete valid window"
        );
    }

    #[test]
    fn test_invalid_duplicate_band_proposal() {
        let entropic = Entropic::new(vec![(0, 0), (0, 1), (0, 2)]);
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 0, 2);
        grid.set_cell(0, 1, 1);
        assert!(
            !entropic.is_valid(&grid, 0, 2, 5),
            "Two lows already - invalid"
        );
    }

    #[test]
    fn test_invalid_add_same_band_proposal() {
        let entropic = Entropic::new(vec![(0, 0), (0, 1), (0, 2)]);
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 0, 2);
        grid.set_cell(0, 1, 5);
        assert!(
            !entropic.is_valid(&grid, 0, 2, 1),
            "Trying to add another low - invalid"
        );
    }

    #[test]
    fn test_valid_long_line_multiple_windows() {
        let entropic = Entropic::new(vec![(0, 0), (0, 1), (0, 2), (0, 3), (0, 4), (0, 5)]);
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 0, 1); // L
        grid.set_cell(0, 1, 5); // M
        grid.set_cell(0, 2, 7); // H
        grid.set_cell(0, 3, 3); // L
        grid.set_cell(0, 4, 6); // M
        assert!(
            entropic.is_valid(&grid, 0, 5, 9),
            "Should complete all windows validly"
        );
    }

    #[test]
    fn test_invalid_middle_window_violation() {
        let entropic = Entropic::new(vec![(0, 0), (0, 1), (0, 2), (0, 3)]);
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 0, 1); // L
        grid.set_cell(0, 1, 5); // M
        grid.set_cell(0, 3, 4); // M
        assert!(
            !entropic.is_valid(&grid, 0, 2, 2),
            "High digit expected - invalid"
        );
    }

    #[test]
    fn test_basic_get_possibilities() {
        let line = Entropic::new(vec![(1, 1), (1, 2), (1, 3), (1, 4)]);
        let mut grid = SudokuGrid::empty();
        grid.set_cell(1, 1, 1); // Low value
        grid.set_cell(1, 3, 6); // Medium value
        let result = line.get_possibilities(&grid, 1, 3);
        assert_eq!(result.len(), 2);
        assert_eq!(result.get(&(1, 2)).unwrap(), &vec![7, 8, 9]);
        assert_eq!(result.get(&(1, 4)).unwrap(), &vec![1, 2, 3]);
    }
}
