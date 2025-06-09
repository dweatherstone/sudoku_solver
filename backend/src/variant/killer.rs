use serde::{Deserialize, Serialize};

use crate::variant::Variant;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KillerCage {
    cells: Vec<(usize, usize)>,
    sum: u32,
}

impl KillerCage {
    pub fn new(cells: Vec<(usize, usize)>, sum: u32) -> Self {
        KillerCage { cells, sum }
    }
}

impl Variant for KillerCage {
    fn is_valid(&self, grid: &crate::SudokuGrid, row: usize, col: usize, value: u8) -> bool {
        // If (row, col) is not in the cage, just pass
        if !self.cells.contains(&(row, col)) {
            return true;
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

    fn affected_cells(&self) -> Vec<(usize, usize)> {
        self.cells.clone()
    }

    fn name(&self) -> String {
        String::from("Killer Cage")
    }
}
