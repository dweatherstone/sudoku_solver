use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{SudokuVariant, variant::Variant};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Diagonal {
    cells: Vec<(usize, usize)>,
}

impl Diagonal {
    pub fn new(is_positive_diagonal: bool) -> Self {
        let cells = if is_positive_diagonal {
            (0..9).map(|i| (8 - i, i)).collect()
        } else {
            (0..9).map(|i| (i, i)).collect()
        };
        Diagonal { cells }
    }

    pub fn parse(data: &str) -> Option<SudokuVariant> {
        match data.trim().to_lowercase().as_str() {
            "positive" => Some(SudokuVariant::Diagonal(Diagonal::new(true))),
            "negative" => Some(SudokuVariant::Diagonal(Diagonal::new(false))),
            _ => None,
        }
    }
}

impl Variant for Diagonal {
    fn is_valid(&self, grid: &crate::SudokuGrid, row: usize, col: usize, value: u8) -> bool {
        if !self.cells.contains(&(row, col)) {
            return true;
        }
        for &(r, c) in &self.cells {
            if grid.get_cell(r, c) == value {
                return false;
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

        // Check all values are unique
        let mut seen = HashSet::new();
        values.iter().all(|&v| seen.insert(v))
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

        let known_cells: HashSet<u8> = self
            .cells
            .iter()
            .map(|&(r, c)| grid.get_cell(r, c))
            .filter(|&v| v != 0)
            .collect();
        let possible_cells: Vec<u8> = (1..=9).filter(|v| !known_cells.contains(v)).collect();
        let mut possibilities = HashMap::new();
        for &(r, c) in self.cells.iter() {
            if r == row && c == col {
                continue;
            }
            let val = grid.get_cell(r, c);
            if val == 0 {
                possibilities.insert((r, c), possible_cells.clone());
            } else {
                possibilities.insert((r, c), vec![val]);
            }
        }
        possibilities
    }
}

impl std::fmt::Display for Diagonal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.cells[0] == (0, 0) {
            write!(f, "Negative Diagonal")
        } else {
            write!(f, "Positive Diagonal")
        }
    }
}
