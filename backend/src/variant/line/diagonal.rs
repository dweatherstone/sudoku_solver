use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
    SudokuGrid, SudokuVariant,
    variant::{ALL_POSSIBILITIES, Variant, error::PossibilityResult},
};

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

    fn get_possibilities(&self, grid: &SudokuGrid) -> PossibilityResult {
        let known_cells: HashMap<(usize, usize), u8> = self
            .cells
            .iter()
            .filter_map(|&(row, col)| {
                let val = grid.get_cell(row, col);
                (val != 0).then_some(((row, col), val))
            })
            .collect();
        let used: HashSet<u8> = known_cells.values().copied().collect();

        let poss: Vec<u8> = ALL_POSSIBILITIES
            .iter()
            .copied()
            .filter(|v| !used.contains(v))
            .collect();

        Ok(self
            .cells
            .iter()
            .map(|&cell| {
                if let Some(&v) = known_cells.get(&cell) {
                    (cell, vec![v])
                } else {
                    (cell, poss.clone())
                }
            })
            .collect())
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
